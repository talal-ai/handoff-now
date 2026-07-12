use crate::config::RedactionMode;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RedactionReport {
    pub categories: BTreeMap<String, usize>,
}

pub struct Redactor {
    patterns: Vec<(&'static str, Regex)>,
    strict: bool,
}

impl Default for Redactor {
    fn default() -> Self {
        let defs = [
            (
                "private_key",
                r"(?s)-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----.*?-----END (?:RSA |EC |OPENSSH )?PRIVATE KEY-----",
            ),
            (
                "authorization",
                r"(?i)(authorization\s*[:=]\s*(?:bearer|basic)\s+)[A-Za-z0-9._~+/=-]+",
            ),
            ("anthropic_key", r"sk-ant-[A-Za-z0-9_-]{20,}"),
            ("openai_key", r"sk-[A-Za-z0-9]{32,}"),
            ("github_token", r"gh[pousr]_[A-Za-z0-9]{30,}"),
            ("aws_access_key", r"AKIA[0-9A-Z]{16}"),
            (
                "connection_string",
                r#"(?i)(?:postgres(?:ql)?|mysql|mongodb(?:\+srv)?|redis)://[^\s"']+"#,
            ),
            ("cookie", r"(?i)(?:cookie|set-cookie)\s*:\s*[^\r\n]+"),
            (
                "env_secret",
                r"(?im)^([A-Z][A-Z0-9_]*(?:TOKEN|SECRET|PASSWORD|API_KEY|PRIVATE_KEY)[A-Z0-9_]*\s*=\s*).+$",
            ),
        ];
        Self {
            patterns: defs
                .into_iter()
                .map(|(n, p)| (n, Regex::new(p).unwrap()))
                .collect(),
            strict: false,
        }
    }
}

impl Redactor {
    pub fn for_mode(mode: &RedactionMode) -> Self {
        Self {
            strict: matches!(mode, RedactionMode::Strict),
            ..Self::default()
        }
    }

    pub fn redact(&self, input: &str) -> (String, RedactionReport) {
        let mut output = input.to_owned();
        let mut report = RedactionReport::default();
        for (name, pattern) in &self.patterns {
            let count = pattern.find_iter(&output).count();
            if count > 0 {
                report.categories.insert((*name).to_string(), count);
                output = pattern
                    .replace_all(&output, |caps: &regex::Captures| {
                        if *name == "authorization" || *name == "env_secret" {
                            format!(
                                "{}[REDACTED:{}]",
                                caps.get(1).map(|m| m.as_str()).unwrap_or(""),
                                name
                            )
                        } else {
                            format!("[REDACTED:{}]", name)
                        }
                    })
                    .into_owned();
            }
        }
        if self.strict {
            let (stricter, hits) = redact_high_entropy(&output);
            if hits > 0 {
                report.categories.insert("high_entropy".into(), hits);
                output = stricter;
            }
        }
        (output, report)
    }

    pub fn contains_secret(&self, input: &str) -> bool {
        if self.patterns.iter().any(|(_, p)| p.is_match(input)) {
            return true;
        }
        self.strict && redact_high_entropy(input).1 > 0
    }
}

/// Shannon entropy (bits per char) of a token.
fn shannon_entropy(token: &str) -> f64 {
    let mut counts = std::collections::HashMap::new();
    for c in token.chars() {
        *counts.entry(c).or_insert(0usize) += 1;
    }
    let len = token.chars().count() as f64;
    counts
        .values()
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

/// Redact long, high-entropy, mixed-charset tokens that pattern matching
/// misses (opaque bearer/session tokens). Conservative: requires length,
/// entropy, and character-class diversity so prose and hashes-in-context are
/// left readable. Public model ids like `claude-...` are dominated by dashes
/// and low entropy, so they survive.
fn redact_high_entropy(input: &str) -> (String, usize) {
    let mut hits = 0usize;
    let out = input
        .split_inclusive(|c: char| c.is_whitespace() || matches!(c, '"' | '\'' | ',' | ')' | '('))
        .map(|chunk| {
            let trimmed = chunk.trim_end_matches(|c: char| {
                c.is_whitespace() || matches!(c, '"' | '\'' | ',' | ')' | '(')
            });
            let suffix = &chunk[trimmed.len()..];
            if is_high_entropy_secret(trimmed) {
                hits += 1;
                format!("[REDACTED:high_entropy]{suffix}")
            } else {
                chunk.to_string()
            }
        })
        .collect::<String>();
    (out, hits)
}

fn is_high_entropy_secret(token: &str) -> bool {
    if token.len() < 24 || token.len() > 200 {
        return false;
    }
    if !token.chars().all(|c| c.is_ascii_graphic()) {
        return false;
    }
    let has_lower = token.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = token.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = token.chars().any(|c| c.is_ascii_digit());
    // Require ALL three classes plus high entropy. Public model ids
    // (`claude-...`), UUIDs, and git hashes are lowercase+digits only, so they
    // stay readable; opaque mixed-case bearer/session tokens are caught.
    has_lower && has_upper && has_digit && shannon_entropy(token) >= 3.6
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_credentials() {
        let r = Redactor::default();
        let (out, report) = r.redact("ANTHROPIC_API_KEY=sk-ant-abcdefghijklmnopqrstuvwxyz\nAuthorization: Bearer abc.def.ghi");
        assert!(!out.contains("abcdefghijklmnopqrstuvwxyz"));
        assert!(report.categories.values().sum::<usize>() >= 1);
    }

    #[test]
    fn strict_catches_high_entropy_token_standard_misses() {
        let opaque = "session=xQ7pL2mZ9rV4kT8wB3nF6dH1jC5aY0sE"; // no known prefix
        let standard = Redactor::default();
        let (std_out, _) = standard.redact(opaque);
        assert!(std_out.contains("xQ7pL2mZ9rV4kT8wB3nF6dH1jC5aY0sE"));
        let strict = Redactor::for_mode(&RedactionMode::Strict);
        let (strict_out, report) = strict.redact(opaque);
        assert!(!strict_out.contains("xQ7pL2mZ9rV4kT8wB3nF6dH1jC5aY0sE"));
        assert!(report.categories.contains_key("high_entropy"));
    }

    #[test]
    fn strict_leaves_public_model_ids_and_prose() {
        let strict = Redactor::for_mode(&RedactionMode::Strict);
        let text = "use claude-haiku-4-5-20251001 to summarize the handoff package now";
        let (out, _) = strict.redact(text);
        assert_eq!(out, text);
    }
}
