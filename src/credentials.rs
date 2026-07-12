use anyhow::{Context, Result};
use keyring::Entry;
use std::env;

const SERVICE: &str = "handoff-now";
const ACCOUNT: &str = "anthropic-api-key";

pub fn api_key() -> Option<String> {
    env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            Entry::new(SERVICE, ACCOUNT)
                .ok()?
                .get_password()
                .ok()
                .filter(|v| !v.trim().is_empty())
        })
}

pub fn store_api_key(value: &str) -> Result<()> {
    let value = value.trim();
    if value.is_empty() {
        anyhow::bail!("credential is empty");
    }
    if !value.starts_with("sk-ant-") {
        anyhow::bail!("credential does not look like an Anthropic API key");
    }
    Entry::new(SERVICE, ACCOUNT)?
        .set_password(value)
        .context("store credential in the OS keychain")
}

pub fn delete_api_key() -> Result<()> {
    let entry = Entry::new(SERVICE, ACCOUNT)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(err).context("delete credential from the OS keychain"),
    }
}

pub fn source() -> &'static str {
    if env::var_os("ANTHROPIC_API_KEY").is_some() {
        "environment"
    } else if api_key().is_some() {
        "os-keychain"
    } else {
        "none"
    }
}
