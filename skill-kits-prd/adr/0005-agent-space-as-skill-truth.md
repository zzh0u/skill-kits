# Agent Space is the Skill source of truth

Status: supersedes managed-copy-first behavior for hard-cut v1.

Skill-kits will manage Skills in the directories Agents actually read from, instead of treating Managed Inventory, Deployment Records, or the TOML Registry as the source of truth. `SKILL.md` means enabled and `SKILL.md.disabled` means disabled.

Hard-cut v1 uses native Agent Space scanning plus a TOML Skill Instance Index. The index is a cache for fast CLI/GUI reads, not authoritative state. When disk and index disagree, disk wins and the affected scope is rescanned.

The old `~/.skill-kits/skills` Managed Inventory and `deployments.toml` Project Deployment Registry are legacy state. They must not participate in the core paths for `list`, `status`, `project status`, `enable`, or `disable`. They may be surfaced by `doctor` as legacy data and may be cleaned up by explicit future commands, but hard-cut v1 does not automatically migrate, delete, or rewrite them.

This replaces the managed-inventory-first mental model because Registry-only disable cannot stop Codex, Claude, or Gemini from reading a Skill that still exists in their native Skill directories.
