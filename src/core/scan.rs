use crate::core::Result;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::fs;
use walkdir::WalkDir;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RiskFinding {
    pub severity: RiskSeverity,
    pub rule_id: String,
    pub path: Utf8PathBuf,
    pub line: Option<usize>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RiskSeverity {
    Info,
    Warn,
    High,
}

pub fn scan_skill_dir(skill_dir: &Utf8Path) -> Result<Vec<RiskFinding>> {
    let mut findings = Vec::new();

    for entry in WalkDir::new(skill_dir).follow_links(false) {
        let entry = entry.map_err(|source| std::io::Error::other(source.to_string()))?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = Utf8PathBuf::from_path_buf(entry.path().to_path_buf()).map_err(|path| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("path is not UTF-8: {}", path.display()),
            )
        })?;
        if !is_scannable_file(&path) {
            continue;
        }

        let text = fs::read_to_string(entry.path())?;
        findings.extend(scan_markdown(&path, &text));
    }

    Ok(findings)
}

fn is_scannable_file(path: &Utf8Path) -> bool {
    let file_name = path.file_name().unwrap_or_default();
    file_name == "SKILL.md"
        || file_name == "SKILL.md.disabled"
        || matches!(
            path.extension().unwrap_or_default(),
            "sh" | "bash" | "zsh" | "fish"
        )
}

fn scan_markdown(path: &Utf8Path, text: &str) -> Vec<RiskFinding> {
    let mut findings = Vec::new();
    let lines: Vec<_> = text.lines().collect();
    let file_is_shell = is_shell_file(path);
    let mut in_shell_fence = false;

    for (idx, line) in lines.iter().enumerate() {
        if let Some(shell_fence) = shell_fence_state(line) {
            in_shell_fence = shell_fence;
            continue;
        }

        let lower = line.to_ascii_lowercase();
        let shell_context = file_is_shell || in_shell_fence;

        if (lower.contains("curl") || lower.contains("wget"))
            && lower.contains('|')
            && lower.contains("sh")
        {
            findings.push(finding(
                path,
                idx,
                RiskSeverity::High,
                "remote-shell-pipe",
                "network pipe to shell",
            ));
        }
        if lower.contains("rm -rf") {
            findings.push(finding(
                path,
                idx,
                RiskSeverity::High,
                "destructive-delete",
                "destructive filesystem command",
            ));
        }
        if lower.contains("sudo") || lower.contains("chmod +x") {
            findings.push(finding(
                path,
                idx,
                RiskSeverity::Warn,
                "privilege-change",
                "privilege-changing instruction",
            ));
        }
        if lower.contains("token")
            || lower.contains("secret")
            || lower.contains("credential")
            || lower.contains("env")
            || lower.contains("api_key")
            || lower.contains("api key")
        {
            findings.push(finding(
                path,
                idx,
                RiskSeverity::High,
                "credential-access",
                "token, secret, credential, or env access",
            ));
        }
        if [
            "http://",
            "https://",
            "npm install",
            "pip install",
            "curl ",
            "wget ",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
        {
            findings.push(finding(
                path,
                idx,
                RiskSeverity::Warn,
                "network-fetch",
                "network fetch instruction",
            ));
        }
        if is_unknown_binary_execution(line, shell_context) {
            findings.push(finding(
                path,
                idx,
                RiskSeverity::Warn,
                "unknown-binary",
                "unknown binary execution reference",
            ));
        }
    }

    findings
}

fn is_shell_file(path: &Utf8Path) -> bool {
    matches!(
        path.extension().unwrap_or_default(),
        "sh" | "bash" | "zsh" | "fish"
    )
}

fn shell_fence_state(line: &str) -> Option<bool> {
    let trimmed = line.trim_start();
    let marker = if trimmed.starts_with("```") {
        "```"
    } else if trimmed.starts_with("~~~") {
        "~~~"
    } else {
        return None;
    };
    let language = trimmed
        .trim_start_matches(marker)
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();

    if language.is_empty() {
        Some(false)
    } else {
        Some(matches!(
            language.as_str(),
            "sh" | "shell" | "bash" | "zsh" | "fish" | "console" | "terminal"
        ))
    }
}

fn is_unknown_binary_execution(line: &str, shell_context: bool) -> bool {
    if !shell_context {
        return false;
    }

    let mut words = line.split_whitespace().peekable();
    while let Some(word) = words.next() {
        let word = trim_shell_token(word);
        let lower = word.to_ascii_lowercase();

        if is_shell_prompt(word) || is_env_assignment(word) || is_command_wrapper(&lower) {
            continue;
        }

        if lower.starts_with("bash<(")
            || lower.starts_with("bash <(")
            || lower.starts_with("sh<(")
            || lower.starts_with("sh <(")
        {
            return true;
        }

        if is_unknown_binary_token(&lower) {
            return true;
        }

        if lower == "bash" || lower == "sh" {
            return words
                .peek()
                .map(|next| trim_shell_token(next).starts_with("<("))
                .unwrap_or(false);
        }

        return false;
    }

    false
}

fn trim_shell_token(token: &str) -> &str {
    token.trim_matches(|ch: char| {
        matches!(
            ch,
            '"' | '\'' | '`' | ';' | '&' | '|' | '(' | ')' | '[' | ']' | '{' | '}'
        )
    })
}

fn is_shell_prompt(token: &str) -> bool {
    matches!(token, "$" | "#" | "%" | ">")
}

fn is_env_assignment(token: &str) -> bool {
    let Some((name, value)) = token.split_once('=') else {
        return false;
    };

    !name.is_empty()
        && !value.is_empty()
        && name
            .chars()
            .all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
        && name
            .chars()
            .next()
            .is_some_and(|ch| ch == '_' || ch.is_ascii_alphabetic())
}

fn is_command_wrapper(token: &str) -> bool {
    matches!(
        token,
        "sudo" | "env" | "command" | "exec" | "nohup" | "time" | "wine"
    )
}

fn is_unknown_binary_token(token: &str) -> bool {
    token.starts_with("./")
        || token.starts_with("../")
        || token.ends_with(".exe")
        || token.contains(".exe/")
}

fn finding(
    path: &Utf8Path,
    line_index: usize,
    severity: RiskSeverity,
    rule_id: &str,
    message: &str,
) -> RiskFinding {
    RiskFinding {
        severity,
        rule_id: rule_id.to_string(),
        path: path.to_path_buf(),
        line: Some(line_index + 1),
        message: message.to_string(),
    }
}

pub fn scan_skill_text(path: &Utf8Path, text: &str) -> Vec<RiskFinding> {
    scan_markdown(path, text)
}
