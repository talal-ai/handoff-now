use crate::{
    config::Config,
    credentials,
    state::{atomic_write, atomic_write_json},
};
use anyhow::{bail, Context, Result};
use chrono::Utc;
use serde_json::{json, Map, Value};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn claude_settings() -> Result<PathBuf> {
    Ok(Config::claude_dir()?.join("settings.json"))
}

pub fn install() -> Result<PathBuf> {
    let root = Config::user_root()?;
    let bin = root.join("bin");
    fs::create_dir_all(&bin)?;
    let source = env::current_exe()?;
    let name = if cfg!(windows) {
        "handoff-now.exe"
    } else {
        "handoff-now"
    };
    let stable = bin.join(name);
    if source != stable {
        fs::copy(&source, &stable)
            .with_context(|| format!("copy binary to {}", stable.display()))?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&stable, fs::Permissions::from_mode(0o700))?;
    }

    let config_path = root.join("config.json");
    if !config_path.exists() {
        atomic_write_json(&config_path, &Config::default())?;
    }

    let settings_path = claude_settings()?;
    fs::create_dir_all(settings_path.parent().unwrap())?;
    let mut settings: Value = if settings_path.exists() {
        serde_json::from_slice(&fs::read(&settings_path)?).context("parse Claude settings")?
    } else {
        Value::Object(Map::new())
    };
    let object = settings
        .as_object_mut()
        .context("Claude settings root must be an object")?;
    let current = object.get("statusLine").cloned().unwrap_or(Value::Null);
    let already_ours = current
        .get("command")
        .and_then(Value::as_str)
        .is_some_and(|s| s.contains("handoff-now") && s.contains("statusline"));
    let install_path = root.join("install.json");
    if !already_ours {
        let stamp = Utc::now().format("%Y%m%dT%H%M%SZ");
        if settings_path.exists() {
            fs::copy(
                &settings_path,
                root.join(format!("settings.{stamp}.backup.json")),
            )?;
        }
        atomic_write_json(
            &install_path,
            &json!({
                "schemaVersion": 1,
                "installedAt": Utc::now().to_rfc3339(),
                "settingsPath": settings_path,
                "previousStatusLine": current,
                "binaryPath": stable
            }),
        )?;
        let command = format!("\"{}\" statusline", stable.display());
        object.insert(
            "statusLine".into(),
            json!({"type":"command","command":command}),
        );
        atomic_write_json(&settings_path, &settings)?;
    }
    Ok(stable)
}

pub fn uninstall() -> Result<()> {
    let root = Config::user_root()?;
    let install_path = root.join("install.json");
    if !install_path.exists() {
        return Ok(());
    }
    let install: Value = serde_json::from_slice(&fs::read(&install_path)?)?;
    let settings_path = claude_settings()?;
    let mut settings: Value = if settings_path.exists() {
        serde_json::from_slice(&fs::read(&settings_path)?)?
    } else {
        json!({})
    };
    let object = settings
        .as_object_mut()
        .context("Claude settings root must be an object")?;
    let still_ours = object
        .get("statusLine")
        .and_then(|v| v.get("command"))
        .and_then(Value::as_str)
        .is_some_and(|s| s.contains("handoff-now") && s.contains("statusline"));
    if !still_ours {
        bail!("statusLine changed after handoff-now setup; refusing to overwrite the user's newer setting");
    }
    let previous = install
        .get("previousStatusLine")
        .cloned()
        .unwrap_or(Value::Null);
    if previous.is_null() {
        object.remove("statusLine");
    } else {
        object.insert("statusLine".into(), previous);
    }
    atomic_write_json(&settings_path, &settings)?;
    atomic_write(
        &root.join("uninstalled.txt"),
        format!(
            "Uninstalled at {}. State retained for recovery.\n",
            Utc::now().to_rfc3339()
        )
        .as_bytes(),
    )?;
    fs::remove_file(install_path)?;
    Ok(())
}

pub fn doctor() -> Result<Value> {
    let root = Config::user_root()?;
    let config = Config::load_or_default(&root.join("config.json"), &root.join("diagnostics.log"));
    let settings_path = claude_settings()?;
    let settings: Value = fs::read(&settings_path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    let status_installed = settings
        .pointer("/statusLine/command")
        .and_then(Value::as_str)
        .is_some_and(|s| s.contains("handoff-now"));
    let git = std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    Ok(json!({
        "ok": status_installed && config.validate().is_ok(),
        "root": root,
        "statusLineInstalled": status_installed,
        "configValid": config.validate().is_ok(),
        "gitAvailable": git,
        "semanticCredentialSource": credentials::source(),
        "telemetry": false
    }))
}

pub fn print_config_path() -> Result<PathBuf> {
    Ok(Config::user_root()?.join("config.json"))
}

pub fn stable_binary_path() -> Result<PathBuf> {
    let name = if cfg!(windows) {
        "handoff-now.exe"
    } else {
        "handoff-now"
    };
    Ok(Config::user_root()?.join("bin").join(name))
}

pub fn settings_path_for_tests(home: &Path) -> PathBuf {
    home.join(".claude").join("settings.json")
}
