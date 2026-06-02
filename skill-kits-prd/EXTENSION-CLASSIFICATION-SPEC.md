# Extension Classification and Scan Boundary Spec

Status: frozen direction for future implementation planning on 2026-06-02.

## Purpose

Skill-kits must separate native Agent Space Skills from plugin-provided runtime capabilities. The current behavior mixes `~/.codex/plugins/cache/.../skills` into the Skill table, which makes plugin package contents look like user-toggleable Skills. This is misleading.

Future classification and GUI work should follow this spec.

## Decision

Skill-kits has two distinct product surfaces:

```text
Skills
Native Agent Space Skill instances that Agents read as user/project Skills.
These may be enabled or disabled by renaming SKILL.md / SKILL.md.disabled when writable.

Plugins / Runtime Capabilities
Agent plugin packages and the capabilities they provide, including plugin-bundled SKILL.md files.
These are read-only from the Skill toggle surface and must be managed through plugin enablement, not Skill file rename.
```

`~/.codex/plugins/cache/.../skills/<name>/SKILL.md` is not a native Codex Skill instance. It is a Skill document bundled inside a Codex plugin package.

## Core Principle

```text
If a file is part of a plugin package, it is plugin content.
If a file is in an Agent's native user/project Skill directory, it is a Skill instance.
```

Do not classify something as a toggleable Skill merely because it contains `SKILL.md`.

## HarnessKit Reference

HarnessKit models all managed things as `Extension`, but separates their kinds:

```rust
ExtensionKind::Skill
ExtensionKind::Mcp
ExtensionKind::Plugin
ExtensionKind::Hook
ExtensionKind::Cli
```

The important pattern is that each Agent adapter declares separate scan surfaces:

```rust
fn skill_dirs(&self) -> Vec<PathBuf>
fn plugin_dirs(&self) -> Vec<PathBuf>
fn project_skill_dirs(&self) -> Vec<String>
```

For Codex, HarnessKit scans native Skills from:

```text
~/.agents/skills
~/.codex/skills
<project>/.agents/skills
```

It treats Codex plugins separately through the plugin surface:

```text
~/.codex/plugins
~/.codex/plugins/cache/{marketplace}/{plugin}/{version}/.codex-plugin/plugin.json
```

The plugin manifest can point to `./skills/`, but those Skill docs remain plugin-provided capabilities, not native Skill instances.

## Classification Model

Skill-kits should introduce or emulate these categories.

### Native Skill Instance

A physical Skill directory in a native Agent Space.

Examples:

```text
~/.agents/skills/<skill>/SKILL.md
~/.agents/skills/<skill>/SKILL.md.disabled
~/.codex/skills/<skill>/SKILL.md
<project>/.agents/skills/<skill>/SKILL.md
```

Properties:

- appears in the main Skill table
- one row equals one physical Skill directory
- status is derived from `SKILL.md` / `SKILL.md.disabled`
- may be toggled when source and filesystem are writable
- can be a real directory or a symlink target reachable through a native Agent Space directory

### Plugin

An Agent plugin package.

Examples:

```text
~/.codex/plugins/cache/openai-curated/github/<hash>/.codex-plugin/plugin.json
~/.codex/plugins/cache/openai-bundled/browser/<version>/.codex-plugin/plugin.json
```

Properties:

- appears in a Plugins or Runtime view, not the main native Skill table
- enablement is controlled through plugin-level configuration or plugin manager behavior
- package contents should not be modified by native Skill toggle operations
- plugin-bundled `skills/` entries are children/capabilities of the plugin

### Plugin-Provided Skill

A `SKILL.md` bundled inside a plugin package.

Examples:

```text
~/.codex/plugins/cache/openai-curated/github/<hash>/skills/github/SKILL.md
~/.codex/plugins/cache/openai-bundled/browser/<version>/skills/control-in-app-browser/SKILL.md
```

Properties:

- visible only as plugin-provided runtime capability
- read-only from Skill-kits' native Skill toggle UI
- grouped under its parent plugin when possible
- not counted as a native Agent Space Skill instance
- not eligible for `SKILL.md -> SKILL.md.disabled` toggle

### Skills-Manager Linked Skill

A native Codex Skill entry where `~/.codex/skills/<skill>` is a symlink to another store such as `~/.skills-manager/skills/<skill>`.

Properties:

- classified as a native Skill instance if discovered through `~/.codex/skills` or `~/.agents/skills`
- source origin should show that it is a symlink or external manager link
- still belongs in the Skill table because Codex can read it through a native Skill directory
- should not cause Skill-kits to scan the whole `~/.skills-manager/skills` root unless that root is explicitly configured as an Agent Space

### Vendor or Import Cache

A non-native import/cache directory that may contain Skill-shaped folders.

Properties:

- not scanned as native Skills by default
- may be shown in a diagnostics or legacy/import surface
- not eligible for native Skill toggle

## Codex Scan Boundaries

### Native Codex Skill Scan

For Codex, native Skill scanning should cover:

```text
~/.agents/skills
~/.codex/skills
<project>/.agents/skills
```

`~/.agents/skills` should be preferred when both paths exist, because current Codex documentation treats it as the canonical user-scope Skill path. `~/.codex/skills` remains supported for backward compatibility.

### Codex Plugin Scan

Codex plugin scanning should cover plugin packages, not native Skill instances:

```text
~/.codex/plugins
~/.codex/plugins/cache
```

The scanner should read plugin manifests such as:

```text
.codex-plugin/plugin.json
```

If a plugin manifest contains:

```json
{
  "skills": "./skills/"
}
```

Skill-kits should classify those entries as plugin-provided capabilities under the plugin.

### Paths Not in Native Defaults

These paths must not be native Codex Skill defaults:

```text
~/.codex/plugins/cache
~/.codex/vendor_imports
~/.skills-manager/skills
```

They may appear through other surfaces:

- `~/.codex/plugins/cache` as plugin packages and plugin-provided capabilities
- `~/.codex/vendor_imports` as vendor/import diagnostics
- `~/.skills-manager/skills` only through symlinks inside native Skill dirs, or explicit custom Agent configuration

## Toggle Rules

Native Skill toggle:

```text
Disable: SKILL.md -> SKILL.md.disabled
Enable:  SKILL.md.disabled -> SKILL.md
```

Allowed only when:

- the item is classified as `Native Skill Instance`
- the selected instance has a valid toggle state
- the path is writable
- the source is not plugin/cache/vendor

Blocked when:

- the item is classified as `Plugin`
- the item is classified as `Plugin-Provided Skill`
- the item lives under `~/.codex/plugins/cache`
- the item lives under vendor/import cache
- both `SKILL.md` and `SKILL.md.disabled` are present
- neither toggle file is present

Plugin toggle must not use `SKILL.md.disabled`. It should use plugin-level enablement where supported. If plugin-level enablement is not implemented, show plugin entries as read-only.

## GUI Requirements

### Skill View

The main Skill view should show only native Skill instances by default.

Recommended columns:

```text
Skill
Agent
Scope
Status
Source
Origin
Updated
```

`Source` examples:

```text
Codex user
Codex legacy user
Project
Claude user
Gemini user
```

`Origin` examples:

```text
Native
Symlink
Skills-manager link
Custom path
```

The Skill view may include an optional filter to show read-only non-native items, but it must not present plugin-provided capabilities as normal toggleable Skills.

### Plugin / Runtime View

Plugin packages should be shown separately.

Recommended columns:

```text
Plugin
Provider
Agent
Status
Capabilities
Path
Updated
```

Plugin detail should show:

- manifest path
- package path
- bundled skills
- commands
- agents
- apps/assets when known
- whether Skill-kits can toggle the plugin

### Inspector Copy

When a user selects a plugin-provided Skill, the Inspector should say:

```text
This Skill is bundled by a Codex plugin. It is not a native Agent Space Skill and cannot be enabled or disabled by renaming SKILL.md.
```

When a user selects a skills-manager symlinked Skill, the Inspector should say:

```text
This Skill is visible to Codex through a native Skill directory symlink.
```

## CLI Requirements

`skill-kits list` should default to native Skill instances.

Add or reserve explicit listing shapes:

```text
skill-kits list --kind skill
skill-kits list --kind plugin
skill-kits list --kind runtime-capability
skill-kits list --all
```

Native toggles:

```text
skill-kits enable <native-skill-instance>
skill-kits disable <native-skill-instance>
```

Plugin toggles, if implemented later:

```text
skill-kits plugin enable <plugin-id>
skill-kits plugin disable <plugin-id>
```

Native Skill commands must reject plugin-provided Skill ids with an explicit error:

```text
This is a plugin-provided Skill. Disable or uninstall the parent plugin instead.
```

## Data Model Implications

The existing `SkillInstanceSourceKind` should not collapse plugin content into normal Skill instances.

Recommended split:

```rust
enum InventoryItemKind {
    NativeSkillInstance,
    Plugin,
    PluginProvidedSkill,
    Mcp,
    Hook,
    Cli,
    VendorImport,
}
```

If the implementation keeps a separate `SkillInstance` model, then `SkillInstance` should represent native Skills only. Plugin-provided Skills should use another model such as:

```rust
pub struct PluginRuntimeCapability {
    pub id: String,
    pub parent_plugin_id: String,
    pub name: String,
    pub kind: RuntimeCapabilityKind,
    pub path: Utf8PathBuf,
    pub read_only: bool,
}
```

## Migration From Current Skill-kits Behavior

Current behavior includes `~/.codex/plugins/cache` and `~/.codex/vendor_imports` in Codex built-in global Skill dirs. Future implementation should remove those from native Skill scanning.

Migration steps:

1. Update Codex built-in agent defaults:

   ```text
   native skill dirs: ~/.agents/skills, ~/.codex/skills
   plugin dirs: ~/.codex/plugins, ~/.codex/plugins/cache
   vendor/import dirs: diagnostics only
   ```

2. Keep existing user custom paths untouched.

3. Rebuild the Skill Instance Index from native skill dirs.

4. Add a plugin/runtime inventory scan for plugin packages.

5. Update GUI filters and Inspector copy so plugin-provided capabilities no longer appear as toggleable Skills.

6. Update tests to assert that `~/.codex/plugins/cache/.../skills/foo/SKILL.md` is not classified as a native Skill instance.

## Acceptance Criteria

- Codex native Skill scan includes `~/.agents/skills`, `~/.codex/skills`, and project `.agents/skills`.
- Codex native Skill scan does not include `~/.codex/plugins/cache`.
- Plugin packages under `~/.codex/plugins/cache` are discoverable as plugins or runtime capabilities.
- Plugin-provided `SKILL.md` files are grouped under their parent plugin.
- Native `enable` / `disable` rejects plugin-provided Skills.
- GUI Skill table does not mix plugin runtime capabilities with user-toggleable native Skills by default.
- Skills-manager symlinked Skills remain visible when they are linked through a native Skill directory.
- `~/.skills-manager/skills` is not scanned as a Codex native Skill root unless explicitly configured.

## Non-Goals

- Do not implement plugin marketplace management in this spec.
- Do not delete or mutate Codex plugin cache contents.
- Do not uninstall skills-manager.
- Do not migrate user symlinks automatically.
- Do not change HarnessKit code; it is reference only.

## Open Questions

- Should Plugin / Runtime appear as a new top-level GUI view or a filter inside the existing Skill view?
- Should Skill-kits support plugin enable/disable for Codex in v0.1, or show plugins read-only first?
- Should `~/.agents/skills` become the first displayed Codex user Skill root even when `~/.codex/skills` has more existing entries?
- Should native Skill scanning follow symlinks for metadata/hash reads while preserving the visible path as the Agent-readable path?
