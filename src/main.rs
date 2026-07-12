use anyhow::{bail, Context, Result};
use handoff_now::{
    config::Config,
    credentials,
    engine::{handle_hook, handle_statusline, read_stdin, snapshot_session},
    setup,
    state::StateStore,
};
use serde_json::json;
use std::env;

fn main() {
    if let Err(err) = run() {
        eprintln!("handoff-now: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("statusline") => print!("{}", handle_statusline(&read_stdin()?)?),
        Some("hook") => {
            let _declared_event = args.next();
            println!(
                "{}",
                serde_json::to_string(&handle_hook(&read_stdin()?)?.0)?
            );
        }
        Some("setup") => println!(
            "Installed handoff-now watcher at {}",
            setup::install()?.display()
        ),
        Some("uninstall") => {
            setup::uninstall()?;
            println!("Restored the previous Claude status line. Recovery data was retained.");
        }
        Some("doctor") => println!("{}", serde_json::to_string_pretty(&setup::doctor()?)?),
        Some("configure") => println!(
            "Edit {} and run `handoff-now doctor`.",
            setup::print_config_path()?.display()
        ),
        Some("credential") => credential(args.next().as_deref())?,
        Some("status") => status()?,
        Some("now") => {
            let id = args
                .next()
                .or_else(latest_session_id)
                .context("no session id available; pass one explicitly")?;
            println!(
                "Handoff created at {}",
                snapshot_session(&id, "manual /handoff-now:now")?.display()
            );
        }
        Some("resume") => {
            let id = args
                .next()
                .or_else(latest_session_id)
                .context("no session id available")?;
            resume(&id)?;
        }
        Some("snapshot-session") => {
            let id = args.next().context("snapshot-session requires an id")?;
            let _ = snapshot_session(&id, "automatic threshold snapshot")?;
        }
        Some("version") | Some("--version") | Some("-V") => {
            println!("handoff-now {}", env!("CARGO_PKG_VERSION"))
        }
        Some("help") | Some("--help") | Some("-h") | None => print_help(),
        Some(other) => bail!("unknown command `{other}`; run handoff-now help"),
    }
    Ok(())
}

fn store() -> Result<StateStore> {
    Ok(StateStore::new(Config::user_root()?))
}
fn latest_session_id() -> Option<String> {
    store()
        .ok()?
        .list()
        .ok()?
        .first()
        .map(|s| s.session_id.clone())
}

fn status() -> Result<()> {
    let sessions = store()?.list()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({"sessions": sessions}))?
    );
    Ok(())
}

fn resume(id: &str) -> Result<()> {
    let state = store()?.load(id)?.context("session not found")?;
    let handoff = state
        .final_handoff_path
        .context("session has no handoff yet")?;
    if !handoff.is_file() {
        bail!("handoff file does not exist: {}", handoff.display());
    }
    println!("Resume this task using the verified handoff package at `{}`. Verify factual claims against Git and EVENTS.jsonl, preserve the user's constraints, and continue from the first unresolved action.", handoff.parent().unwrap().display());
    Ok(())
}

fn print_help() {
    println!(
        r#"handoff-now - quota-aware crash-safe handoffs for Claude Code

USAGE:
  handoff-now setup
  handoff-now status
  handoff-now now [SESSION_ID]
  handoff-now resume [SESSION_ID]
  handoff-now doctor
  handoff-now configure
  handoff-now credential store|delete|status
  handoff-now uninstall

Internal plugin commands: statusline, hook, snapshot-session.
"#
    );
}

fn credential(action: Option<&str>) -> Result<()> {
    match action {
        Some("store") => {
            use std::io::{IsTerminal, Read};
            if std::io::stdin().is_terminal() {
                bail!("pipe the API key through stdin to avoid command-line history; do not pass it as an argument");
            }
            let mut value = String::new();
            std::io::stdin().read_to_string(&mut value)?;
            credentials::store_api_key(&value)?;
            println!("Stored the Anthropic API key in the operating-system credential store.");
        }
        Some("delete") => {
            credentials::delete_api_key()?;
            println!("Removed the handoff-now API credential.");
        }
        Some("status") => println!("Semantic credential source: {}", credentials::source()),
        _ => bail!("credential requires store, delete, or status"),
    }
    Ok(())
}
