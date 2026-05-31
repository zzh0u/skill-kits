# Skill-kits v0.1 macOS single binary checklist

This checklist is for a local macOS release build of the `skill-kits` binary.

## Verify

Run from the repository root:

```bash
rtk /opt/homebrew/bin/cargo fmt --all --check
rtk /opt/homebrew/bin/cargo clippy --all-targets --all-features -- -D warnings
rtk /opt/homebrew/bin/cargo test
```

## Build

```bash
rtk /opt/homebrew/bin/cargo build --release
```

The release binary is:

```text
target/release/skill-kits
```

## Runtime

The v0.1 macOS release is a single Rust binary. It must not require a Node.js or
Python runtime to list, install, deploy, adopt, scan, run doctor checks, or open
the native GUI.
