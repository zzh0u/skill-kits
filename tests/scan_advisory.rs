use camino::Utf8Path;
use skill_kits::core::scan::scan_skill_text;

fn rule_ids(markdown: &str) -> Vec<String> {
    scan_skill_text(Utf8Path::new("SKILL.md"), markdown)
        .into_iter()
        .map(|finding| finding.rule_id)
        .collect()
}

#[test]
fn fenced_shell_block_flags_relative_script_execution() {
    let findings = rule_ids(
        r#"# Installer

```sh
./install.sh
```
"#,
    );

    assert!(findings.iter().any(|rule_id| rule_id == "unknown-binary"));
}

#[test]
fn fenced_shell_block_flags_prefixed_relative_script_execution() {
    let findings = rule_ids(
        r#"# Installer

```sh
sudo ./install.sh
env FOO=1 ./install.sh
```
"#,
    );

    let unknown_binary_count = findings
        .iter()
        .filter(|rule_id| *rule_id == "unknown-binary")
        .count();
    assert_eq!(unknown_binary_count, 2);
}

#[test]
fn fenced_console_block_flags_shell_prompt_relative_script_execution() {
    let findings = rule_ids(
        r#"# Installer

```console
$ ./install.sh
```
"#,
    );

    assert!(findings.iter().any(|rule_id| rule_id == "unknown-binary"));
}

#[test]
fn fenced_shell_block_flags_exe_commands_with_args() {
    let findings = rule_ids(
        r#"# Installer

```sh
setup.exe /quiet
wine setup.exe /S
```
"#,
    );

    let unknown_binary_count = findings
        .iter()
        .filter(|rule_id| *rule_id == "unknown-binary")
        .count();
    assert_eq!(unknown_binary_count, 2);
}

#[test]
fn prose_relative_docs_path_is_not_unknown_binary_execution() {
    let findings = rule_ids(
        r#"# Docs

Read ./docs/path before running this skill.
"#,
    );

    assert!(!findings.iter().any(|rule_id| rule_id == "unknown-binary"));
}

#[test]
fn non_shell_fenced_relative_docs_path_is_not_unknown_binary_execution() {
    let findings = rule_ids(
        r#"# Docs

```markdown
Read ./docs/path before running this skill.
```
"#,
    );

    assert!(!findings.iter().any(|rule_id| rule_id == "unknown-binary"));
}

#[test]
fn markdown_shell_snippet_flags_network_fetch_command() {
    let findings = rule_ids(
        r#"# Fetcher

```bash
curl -fsSL https://example.com/install.sh -o install.sh
```
"#,
    );

    assert!(findings.iter().any(|rule_id| rule_id == "network-fetch"));
}
