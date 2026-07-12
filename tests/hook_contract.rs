use assert_cmd::Command;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

fn command(home: &Path) -> Command {
    let mut cmd = Command::cargo_bin("handoff-now").unwrap();
    cmd.env("CLAUDE_CONFIG_DIR", home.join(".claude"));
    cmd
}

fn statusline(home: &Path, cwd: &Path, percentage: f64) -> String {
    let input = json!({
        "session_id":"test-session",
        "cwd": cwd,
        "workspace":{"current_dir":cwd},
        "transcript_path": cwd.join("transcript.jsonl"),
        "rate_limits":{"five_hour":{"used_percentage":percentage,"resets_at":2000000000}}
    });
    let output = command(home)
        .arg("statusline")
        .write_stdin(input.to_string())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).unwrap()
}

fn hook(home: &Path, cwd: &Path, event: &str, extra: Value) -> Value {
    let mut input = json!({
        "session_id":"test-session",
        "cwd":cwd,
        "transcript_path":cwd.join("transcript.jsonl"),
        "hook_event_name":event
    });
    input
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let output = command(home)
        .args(["hook", event])
        .write_stdin(input.to_string())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice(&output).unwrap()
}

fn assert_integrity(artifact: &Path) {
    let manifest: Value =
        serde_json::from_slice(&fs::read(artifact.join("integrity.json")).unwrap()).unwrap();
    for (name, expected) in manifest.get("files").and_then(Value::as_object).unwrap() {
        let actual = hex::encode(Sha256::digest(fs::read(artifact.join(name)).unwrap()));
        assert_eq!(
            &actual,
            expected.as_str().unwrap(),
            "integrity mismatch for {name}"
        );
    }
}

#[test]
fn normal_band_does_not_block_tools() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("transcript.jsonl"), "").unwrap();
    assert!(statusline(home.path(), project.path(), 84.0).contains("Normal"));
    let result = hook(
        home.path(),
        project.path(),
        "PreToolUse",
        json!({"tool_name":"Write","tool_input":{"file_path":project.path().join("src.rs")}}),
    );
    assert_eq!(result, json!({}));
}

#[test]
fn emergency_band_denies_source_writes() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("transcript.jsonl"), "").unwrap();
    statusline(home.path(), project.path(), 91.0);
    let result = hook(
        home.path(),
        project.path(),
        "PreToolUse",
        json!({"tool_name":"Write","tool_input":{"file_path":project.path().join("src.rs")}}),
    );
    assert_eq!(
        result
            .pointer("/hookSpecificOutput/permissionDecision")
            .and_then(Value::as_str),
        Some("deny")
    );
}

#[test]
fn hard_stop_returns_continue_false() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("transcript.jsonl"), "").unwrap();
    statusline(home.path(), project.path(), 99.0);
    let result = hook(
        home.path(),
        project.path(),
        "Stop",
        json!({"last_assistant_message":"still working"}),
    );
    assert_eq!(result.get("continue").and_then(Value::as_bool), Some(false));
    assert!(project
        .path()
        .join(".handoff-now/test-session/HANDOFF.md")
        .is_file());
    assert_integrity(&project.path().join(".handoff-now/test-session"));
}

#[test]
fn stop_failure_recovers_without_model() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("transcript.jsonl"), "").unwrap();
    statusline(home.path(), project.path(), 20.0);
    let result = hook(
        home.path(),
        project.path(),
        "StopFailure",
        json!({"error":"rate_limit","error_details":"429"}),
    );
    assert!(result
        .get("systemMessage")
        .and_then(Value::as_str)
        .unwrap()
        .contains("recovered"));
    assert!(project
        .path()
        .join(".handoff-now/test-session/HANDOFF.md")
        .is_file());
}

#[test]
fn setup_is_idempotent_and_restores_previous_statusline() {
    let home = tempfile::tempdir().unwrap();
    let settings = home.path().join(".claude/settings.json");
    fs::create_dir_all(settings.parent().unwrap()).unwrap();
    let prior_command = if cfg!(windows) {
        "Write-Output old-status"
    } else {
        "printf old-status"
    };
    fs::write(
        &settings,
        serde_json::to_vec_pretty(&json!({
            "statusLine":{"type":"command","command":prior_command},
            "permissions":{"allow":["Read"]}
        }))
        .unwrap(),
    )
    .unwrap();

    command(home.path()).arg("setup").assert().success();
    command(home.path()).arg("setup").assert().success();
    let installed: Value = serde_json::from_slice(&fs::read(&settings).unwrap()).unwrap();
    assert!(installed
        .pointer("/statusLine/command")
        .and_then(Value::as_str)
        .unwrap()
        .contains("handoff-now"));
    assert_eq!(
        installed
            .pointer("/permissions/allow/0")
            .and_then(Value::as_str),
        Some("Read")
    );
    let rendered = statusline(home.path(), home.path(), 20.0);
    assert!(rendered.contains("old-status"));
    assert!(rendered.contains("handoff-now"));

    command(home.path()).arg("uninstall").assert().success();
    let restored: Value = serde_json::from_slice(&fs::read(&settings).unwrap()).unwrap();
    assert_eq!(
        restored
            .pointer("/statusLine/command")
            .and_then(Value::as_str),
        Some(prior_command)
    );
}

#[test]
fn preparation_band_creates_checkpoint_without_blocking() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("transcript.jsonl"), "").unwrap();
    statusline(home.path(), project.path(), 86.0);
    let result = hook(
        home.path(),
        project.path(),
        "PreToolUse",
        json!({"tool_name":"Read","tool_input":{"file_path":project.path().join("README.md")}}),
    );
    assert_eq!(result, json!({}));
    assert!(project
        .path()
        .join(".handoff-now/test-session/HANDOFF.md")
        .is_file());
}

#[test]
fn emergency_allows_only_validated_candidate_write() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("transcript.jsonl"), "").unwrap();
    statusline(home.path(), project.path(), 91.0);

    let stop = hook(
        home.path(),
        project.path(),
        "Stop",
        json!({"stop_hook_active":false,"last_assistant_message":"working"}),
    );
    assert_eq!(stop.get("decision").and_then(Value::as_str), Some("block"));

    let artifact = project.path().join(".handoff-now/test-session");
    let forbidden = hook(
        home.path(),
        project.path(),
        "PreToolUse",
        json!({"tool_name":"Write","tool_input":{"file_path":artifact.join("HANDOFF.md"),"content":"corrupt"}}),
    );
    assert_eq!(
        forbidden
            .pointer("/hookSpecificOutput/permissionDecision")
            .and_then(Value::as_str),
        Some("deny")
    );

    let allowed = hook(
        home.path(),
        project.path(),
        "PreToolUse",
        json!({"tool_name":"Write","tool_input":{"file_path":artifact.join("SUMMARY.candidate.md"),"content":"omitted by journal"}}),
    );
    assert_eq!(
        allowed
            .pointer("/hookSpecificOutput/permissionDecision")
            .and_then(Value::as_str),
        Some("allow")
    );

    let session: Value =
        serde_json::from_slice(&fs::read(artifact.join("SESSION.json")).unwrap()).unwrap();
    let seq = session
        .get("journalSequence")
        .and_then(Value::as_u64)
        .unwrap();
    let candidate = format!("# User Goal\nrecover\n\n# Work Completed\ncheckpoint\n\n# Current State\nsafe\n\n# Remaining Work\nresume\n\n# First Action\nverify\n\nJournal Sequence: {seq}\n");
    fs::write(artifact.join("SUMMARY.candidate.md"), candidate).unwrap();
    let promoted = hook(
        home.path(),
        project.path(),
        "PostToolUse",
        json!({"tool_name":"Write","tool_input":{"file_path":artifact.join("SUMMARY.candidate.md")},"tool_response":"ok"}),
    );
    assert_eq!(
        promoted.get("continue").and_then(Value::as_bool),
        Some(false)
    );
    assert!(artifact.join("SUMMARY.validated").is_file());
    assert!(fs::read_to_string(artifact.join("SUMMARY.md"))
        .unwrap()
        .contains("# User Goal"));
    assert_integrity(&artifact);
}
