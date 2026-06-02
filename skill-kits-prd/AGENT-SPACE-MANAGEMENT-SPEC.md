# Agent Space Skill Management Spec

Status: intent frozen for hard-cut implementation planning on 2026-06-02.

## Goal

Skill-kits must hard-cut from managed-copy deployment to native Agent Space management. The directories Agents actually read from are the only source of truth. A Skill is enabled when the native Skill directory contains `SKILL.md`; it is disabled when the same directory contains `SKILL.md.disabled`.

The old Managed Inventory copy and Project Deployment Registry are legacy state. They must not participate in the core read/write paths for `list`, `status`, `project status`, `enable`, or `disable`.

TOML remains the persistence format. The new persisted model is a Skill Instance index: a cache of native Agent Space scan results, not the source of truth. When the index and disk disagree, disk wins and the affected scope is rescanned.

## Core Principle

```text
Agent reads it there, Skill-kits manages it there.
```

`SKILL.md` and `SKILL.md.disabled` are the toggle truth. A Registry flag, Managed Inventory copy, or Deployment Record must never be treated as sufficient to enable or disable a Skill that an Agent reads from disk.

## Concepts

**Agent Space** is any configured directory tree an enabled Agent reads for Skills. Examples include `~/.codex/skills`, `~/.claude/skills`, `~/.gemini/skills`, and project-local directories such as `<project>/.agents/skills`.

**Skill Instance** is one physical Skill directory discovered in an Agent Space. The product is instance-first: one row equals one `skill_dir`.

**Skill Instance Index** is the TOML cache produced by scanning Agent Spaces. It records native Skill Instances for fast CLI/GUI reads, but it is not authoritative. Disk is authoritative.

**Legacy Managed Inventory** is the old Skill-kits-owned copy area such as `~/.skill-kits/skills`. Hard-cut v1 ignores it for core listing, status, project status, enable, and disable. It may be reported by `doctor` as legacy state, but it is not a primary product surface.

**Legacy Deployment** is any old copy from Managed Inventory into a project Agent Space. Hard-cut v1 does not use Deployment Records to decide status or toggle behavior.

## SkillInstance Model

The implementation should introduce a core/index model equivalent to:

```rust
pub struct SkillInstance {
    pub id: String,
    pub name: String,
    pub agent_id: AgentId,
    pub scope: SkillInstanceScope,
    pub skill_dir: Utf8PathBuf,
    pub enabled_path: Utf8PathBuf,
    pub disabled_path: Utf8PathBuf,
    pub toggle_state: ToggleState,
    pub source_kind: SkillInstanceSourceKind,
    pub writable: bool,
    pub metadata: Option<SkillMetadata>,
    pub content_hash: Option<String>,
    pub updated_at: Option<String>,
}
```

`id` is a stable instance id:

```text
hash(agent_id + scope_key + canonical skill_dir path)
```

It must not include Skill name or content hash, because renaming `SKILL.md` or editing content must not change row identity. Same-name Skills in different Agents, projects, or directories are separate instances.

The core model intentionally omits `stable_id`, `managed`, `baseline_hash`, and deployment metadata from the toggle path. Those concepts belong to legacy reporting, not native instance management.

## Index TOML

Persist scan results in a new TOML file, for example `registry/skill_instances.toml`:

```toml
version = 1
last_scanned_at = "2026-06-02T00:00:00Z"

[[instances]]
id = "..."
agent_id = "codex"
scope = "project"
project_path = "/Users/me/work/app"
name = "reviewer"
skill_dir = "/Users/me/work/app/.agents/skills/reviewer"
enabled_path = "/Users/me/work/app/.agents/skills/reviewer/SKILL.md"
disabled_path = "/Users/me/work/app/.agents/skills/reviewer/SKILL.md.disabled"
toggle_state = "enabled"
source_kind = "project_agent_space"
writable = true
content_hash = "..."
updated_at = "2026-06-02T00:00:00Z"
```

The index supports fast reads and selection. It is not a Registry sync target and does not import, copy, adopt, deploy, redeploy, remove, or mutate Skills.

## Toggle State

Toggle state is derived from the filesystem:

| Files in `skill_dir` | State |
| --- | --- |
| `SKILL.md` only | `Enabled` |
| `SKILL.md.disabled` only | `Disabled` |
| both files | `InvalidBothPresent` |
| neither file, but the index references the directory | `InvalidBothMissing` / stale index |

Disabled is a valid state. Metadata and content hash should be read from `SKILL.md.disabled`, with hash path normalization treating `SKILL.md.disabled` as `SKILL.md` so enable/disable rename does not change content identity.

`InvalidBothPresent` is strict: use the directory name, do not compute normal metadata or content hash, disable toggle, and show both file paths in the Inspector. Hard-cut v1 does not auto-repair this state.

Pure filesystem scanning should not invent both-missing rows. Missing rows only appear as stale-index or doctor diagnostics, not as normal Skill rows.

## Scan Rules

Routine Agent Space scan covers:

- all enabled Agents' configured global Skill directories
- all enabled Agents' project Skill directories inside Recent Projects
- read-only plugin/cache/vendor roots that the Agent configuration declares as visible to the Agent

Routine scan does not run a workspace crawler. HarnessKit-style `Discover Projects(root)` is deferred.

Normal Agent Space roots use immediate-child scanning:

```text
<root>/<skill>/SKILL.md
<root>/<skill>/SKILL.md.disabled
```

Plugin/cache/vendor roots use bounded recursive discovery, maximum depth 4. Discovery stops at the first directory containing `SKILL.md` or `SKILL.md.disabled`. It should skip hidden and common generated/noisy directories such as `node_modules`, `target`, `__pycache__`, `vendor`, `dist`, `build`, `venv`, and `.venv`.

Scans write only the Skill Instance Index. They do not write `skills.toml` or `deployments.toml`.

## Source Kinds

Source kind drives toggle behavior:

| Source kind | Meaning | Toggle |
| --- | --- | --- |
| `AgentSpace` | Writable global Agent Space | Allowed when writable |
| `ProjectAgentSpace` | Writable project Agent Space | Allowed when writable |
| `PluginCache` | Agent-visible plugin cache | Read-only |
| `Vendor` | Agent-visible vendor/import cache | Read-only |

`~/.skills-manager/skills` must not remain in Codex built-in global Agent Space defaults. If shown at all, it is legacy/source information, not a Codex-readable Agent Space unless an Agent config explicitly declares it.

## Toggle Rules

Enable and disable operate only on the selected Skill Instance.

Enable:

```text
SKILL.md.disabled -> SKILL.md
```

Disable:

```text
SKILL.md -> SKILL.md.disabled
```

The operation must not delete the Skill directory, mutate Legacy Managed Inventory, write an enablement flag to Registry, or affect same-name copies in other scopes.

Toggle is blocked when:

- both toggle files are present
- both toggle files are missing
- the instance path is not writable
- the source kind is `PluginCache` or `Vendor`

After toggle, Skill-kits refreshes the affected scope and updates the Skill Instance Index. If a command targets a stale index entry whose path no longer exists, it rescans the relevant scope before returning a not-found or ambiguity error.

## CLI Shape

Core commands use native instances:

- `skill-kits scan` refreshes the Skill Instance Index.
- `skill-kits list` lists native Skill Instances from the index, with stale path recovery.
- `skill-kits status` reports Agent count, global/project instance count, invalid count, read-only count, and last scan time.
- `skill-kits enable <instance-id-or-query>` toggles the selected native instance.
- `skill-kits disable <instance-id-or-query>` toggles the selected native instance.
- `skill-kits project status --project <path>` scans/reads project native Agent Spaces.
- `skill-kits project enable <name-or-id> --agent <agent> --project <path>` toggles a project native instance.
- `skill-kits project disable <name-or-id> --agent <agent> --project <path>` toggles a project native instance.

Old copy-model commands (`install local`, `project deploy`, `project redeploy`, `project remove`, managed-copy import/uninstall) are hidden or return explicit legacy warnings in hard-cut v1. If a future install capability is needed, use new wording such as `add-local --to-agent --scope` and copy directly into a native Agent Space.

## GUI Shape

The primary GUI views are:

```text
Dashboard / Skill / Agent / Project
```

The Skill view shows native Agent Space scan results. Hard-cut v1 is instance-first: one row equals one physical Skill directory.

The Skill view should expose:

- Skill
- Agent
- Scope
- Status
- Source
- Updated

Status values:

- Enabled
- Disabled
- Invalid
- Missing
- Read-only

The Inspector shows:

- real `skill_dir`
- enabled and disabled file paths
- Agent
- Scope
- writable/read-only state
- metadata
- content hash when available
- risk findings when available

Action wording:

- `Scan Agent Spaces` refreshes the Skill Instance Index.
- `Enable` / `Disable` rename the selected instance toggle file only.
- Legacy `Install local`, `Deploy to Project`, `Redeploy`, `Remove`, `Import managed copy`, and `Uninstall managed copy` actions are hidden or replaced with legacy warnings in hard-cut v1.

Disable copy:

```text
Disable changes SKILL.md to SKILL.md.disabled in the Agent Space.
It does not delete the Skill directory.
```

Dashboard primary counts lead with Agent Space instances. Legacy Managed Inventory can appear only as a doctor/legacy warning.

## Tranches

### Tranche 1: Native Skill Instance Index

- Add `SkillInstance` and `SkillInstanceIndex`.
- Scan enabled Agent global dirs and Recent Projects.
- Recognize `SKILL.md.disabled`.
- Persist scan results to TOML.
- Show native instances in CLI and GUI.
- Keep scans copy-free and legacy Registry-free.
- Treat plugin/cache/vendor as read-only.

Tests:

- enabled instance
- disabled instance
- invalid both present
- plugin/cache/vendor read-only
- Recent Projects project scan
- stale index path triggers rescan
- legacy `skills.toml` and `deployments.toml` ignored for list/status/toggle

### Tranche 2: Native Toggle

- Enable/disable through rename only.
- Restrict toggle to writable Agent Space instances.
- Block invalid, missing, plugin/cache/vendor, and read-only paths.
- `project status`, `project enable`, and `project disable` use native project Agent Space scan/index data, not Deployment Records.

Tests:

- disable does not delete directory
- enable/disable does not mutate Legacy Managed Inventory
- toggle does not affect unrelated project/global copies
- rescan preserves disabled state
- same-name project/global instances stay isolated

### Tranche 3: CLI/GUI Productization

- Status column and filters.
- Agent and Scope filters.
- Inspector copy for invalid/read-only state.
- Inline confirmation copy for disable.
- Optional `Discover Projects(root)` entry point.

### Tranche 4: Legacy Cleanup

- Add `doctor` checks for Legacy Managed Inventory and Deployment Records.
- Add optional legacy cleanup commands only after the hard-cut model is stable.
- Keep old registry files readable by old releases by not deleting them automatically.

## Non-goals

- Do not implement `skill-kits run codex`.
- Do not add a scheduler or launcher integration.
- Do not rely on `enabled=false` Registry flags for Agent-visible Skills.
- Do not bulk-copy all Agent Space Skills into Legacy Managed Inventory by default.
- Do not modify plugin/cache/vendor roots by default.
- Do not add SQLite in hard-cut v1.
- Do not automatically migrate, delete, or rewrite old Managed Inventory and Deployment Registry data.
- Do not support MCP, Plugin, Hook, CLI, marketplace, or remote update checks in this hard-cut scope.
