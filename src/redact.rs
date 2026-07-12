use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RedactionReport {
    pub categories: BTreeMap<String, usize>,
}

pub struct Redactor {
    patterns: Vec<(&'static str, Regex)>,
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
        }
    }
}

impl Redactor {
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
        (output, report)
    }

    pub fn contains_secret(&self, input: &str) -> bool {
        self.patterns.iter().any(|(_, p)| p.is_match(input))
    }
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
}
