# Skill-kits GUI Spec v0.1

## Design Position

Skill-kits uses a Linear-inspired monochrome workbench style. The interface should feel like a precise local developer tool: quiet, dense, fast to scan, and trustworthy under repeated daily use.

This is not a marketing canvas. The GUI should borrow Linear's product qualities, not its website formula:

- Use black, white, and gray as the brand system.
- Use surface levels and hairline borders for hierarchy.
- Use compact lists, tables, inspectors, and toolbars as the main UI language.
- Use status color only when it carries operational meaning.
- Avoid hero sections, product screenshot framing, lavender brand accents, gradient atmosphere, and SaaS card grids.

## Scene

A developer opens Skill-kits on macOS to answer concrete questions:

- Which Skills are actually visible in Agent Space?
- Which managed copies are available as install/deploy sources?
- Which project am I looking at?
- Which Agent directories are configured?
- Which Skills are deployed, disabled, outdated, drifted, or invalid?
- What action is safe to take next?

The UI should feel calm enough for long sessions and sharp enough for quick status checks.

## Visual Direction

Name: **Monochrome Local Workbench**

Keywords:

- quiet
- precise
- dense
- local-native
- developer-grade
- inspection-first
- low-noise

Use Linear as an inspiration for hierarchy and restraint, but do not clone Linear's brand.

## Color System

### Strategy

Brand color is monochrome. Black, white, and gray carry the identity.

Status colors are semantic, not brand colors. They should be muted and rare.

Do not use lavender, purple, blue-purple gradients, or saturated chromatic accents as a primary brand signal.

### Dark Theme Tokens

Dark theme is the v0.1 default.

| Token | Hex | Use |
| --- | --- | --- |
| `canvas` | `#08090b` | App background |
| `surface_1` | `#101114` | Sidebar and base panels |
| `surface_2` | `#17191d` | Main raised panels, rows on hover |
| `surface_3` | `#202227` | Selected rows, popovers, active controls |
| `surface_4` | `#2a2d33` | Strong focus surfaces |
| `hairline` | `#25272d` | Default 1px dividers and borders |
| `hairline_strong` | `#363942` | Focused borders and active outlines |
| `ink` | `#f2f3f3` | Primary text |
| `ink_muted` | `#b9bec7` | Secondary text |
| `ink_subtle` | `#858b96` | Metadata, paths, helper text |
| `ink_tertiary` | `#5f6570` | Disabled text |
| `inverse_ink` | `#111216` | Text on light controls |

### Semantic Tokens

| Token | Hex | Use |
| --- | --- | --- |
| `success` | `#67a878` | Enabled, healthy |
| `warning` | `#c5a365` | Outdated, caution |
| `danger` | `#d06b6b` | Destructive action, high risk |
| `info` | `#9ea4ad` | Neutral informational state |
| `focus` | `#e4e6eb` | Keyboard focus ring and selected outline |

Rules:

- `focus` is near-white or cool gray, not lavender.
- Status colors must not fill large surfaces.
- Most badges should use gray surfaces with colored text or a tiny dot.
- Destructive actions use muted red text until confirmation, not bright filled red buttons.

### Light Theme

v0.1 ships dark theme only. If light theme is added later, it should invert the same monochrome structure rather than introduce a new brand color.

## Typography

Use platform-native fonts. Skill-kits is a desktop utility, so native text rendering matters more than branded typography. v0.1 does not bundle custom fonts.

Recommended stack:

- UI text: `-apple-system`, `BlinkMacSystemFont`, `SF Pro`, `Segoe UI`, `system-ui`
- Mono: `JetBrains Mono`, `SF Mono`, `ui-monospace`, `Menlo`

Use mono only for:

- paths
- hashes
- Skill IDs
- commands
- registry snippets

Do not use aggressive negative letter spacing in the product UI. Keep letter spacing at `0`.

### Type Scale

| Token | Size | Weight | Use |
| --- | --- | --- | --- |
| `title` | 20 | 600 | Page title |
| `section` | 15 | 600 | Section headings |
| `body` | 13 | 400 | Default table and panel text |
| `body_strong` | 13 | 500 | Names, selected values |
| `caption` | 12 | 400 | Metadata, timestamps, helper text |
| `mono` | 12 | 400 | Paths, IDs, hashes |
| `button` | 13 | 500 | Button labels |

## Layout System

Use a stable desktop app shell:

```text
┌────────────────────────────────────────────────────────────┐
│ Top bar: scope, project, quick actions, status             │
├──────────────┬──────────────────────────────┬──────────────┤
│ Sidebar      │ Main list / table             │ Inspector    │
│ navigation   │                              │ details      │
└──────────────┴──────────────────────────────┴──────────────┘
```

### App Shell

- Sidebar width: 180 to 220px.
- Inspector width: 320 to 380px.
- Main content owns remaining width.
- Minimum usable desktop width: 960px.
- Use split panes, not nested cards.

### macOS Window Chrome

v0.1 targets macOS first, so the app window must feel native while preserving the dark workbench surface.

Requirements:

- The visible title bar area must not remain the default light macOS chrome when the app content is dark.
- Prefer a full-size content view or transparent title bar that lets the dark top bar visually occupy the title region.
- Keep standard macOS window controls visible and usable.
- Do not replace the window chrome with a custom design that breaks platform resize, move, focus, or accessibility behavior.
- The top bar should carry the app title, active scope, project context, quick actions, and action status without looking like a separate web header.

Acceptance:

- A screenshot of the launched macOS app shows one continuous dark workbench surface from the top edge through the content shell.
- The window title area does not introduce a white strip above the dark interface.
- The app remains movable and resizable through normal macOS window interactions.

### Navigation

Order:

1. Dashboard
2. Skills
3. Agents
4. Projects

The Scope Switcher sits at the lower-left or lower sidebar area. It switches between Global Inventory and Recent Projects.

### Density

Default density should be compact:

- Table row height: 32 to 36px.
- Toolbar height: 40px.
- Sidebar item height: 32px.
- Inspector section spacing: 16px.
- Control padding: 6px vertical, 10px horizontal.

Use space to separate meaning, not to make the UI feel luxurious.

### Scrolling and Overflow

Every primary pane must handle long content without clipping, layout breakage, or hidden actions.

Rules:

- The main table/list region scrolls vertically when rows exceed the available height.
- The main table/list region supports horizontal overflow for long paths, long Skill names, and dense project columns.
- The inspector scrolls independently from the main table when metadata, paths, risk findings, or action guidance exceed the available height.
- Action controls should remain reachable when inspector content is long. Prefer a fixed action area or a clearly separated scroll region.
- The sidebar scope switcher must remain reachable when Recent Projects is long.
- Long paths and identifiers should truncate in compact rows, preserve the full value in hover/copy affordances, and render fully in the inspector when space allows.

Acceptance:

- A Skill with a long description and long filesystem paths can be inspected without losing access to the Controls section.
- A project with many Skill instances can be browsed by mouse wheel or trackpad without resizing the window.
- No pane requires dragging the window larger just to discover hidden content.

## Surface and Borders

Depth is carried by surface levels and hairlines:

- App background: `canvas`.
- Sidebar: `surface_1`.
- Main panel: `canvas` or `surface_1`, depending on density.
- Hover row: `surface_2`.
- Selected row: `surface_3`.
- Inspector: `surface_1` with `hairline` separator.
- Popovers: `surface_3` with `hairline_strong`.

Do not use drop shadows as a primary depth mechanism. If egui rendering makes shadows useful for popovers, keep them extremely subtle.

## Shape

| Token | Radius | Use |
| --- | --- | --- |
| `xs` | 3px | small badges |
| `sm` | 5px | table selections, inline tags |
| `md` | 6px | buttons, inputs |
| `lg` | 8px | popovers, inspector groups |

Avoid large rounded cards. The app should feel crisp, not pillowy.

## Components

### Buttons

Primary actions are monochrome:

- Background: `ink`
- Text: `inverse_ink`
- Radius: `md`
- Height: 28 to 32px

Secondary buttons:

- Background: `surface_2`
- Text: `ink`
- Border: `hairline`

Tertiary buttons:

- Background: transparent
- Text: `ink_muted`
- Hover: `surface_2`

Destructive actions:

- Default text: `danger`
- Confirmation state may use muted danger fill
- Never place a large bright red button in a normal toolbar

### Tables and Lists

Tables are the core component.

Rules:

- First column is the human name.
- Status columns use compact badges.
- Paths and hashes use mono.
- Hover reveals row actions when useful.
- Selected row opens or updates the inspector.
- Empty cells use `-`, not blank space.

Workbench table behavior:

- Rows are selected as whole rows, not as separate clickable cells.
- The selected row uses `surface_3`; hover uses `surface_2`.
- Header labels remain visually quieter than row content and should stay visible when the row list scrolls, where egui implementation allows it.
- Row height stays within 32 to 36px in default density.
- Status values render as badges or badge-like inline indicators, not plain unstyled strings.
- Paths, Skill IDs, hashes, and command snippets use mono styling.
- Long cell values truncate in the row and expose the full value through inspector detail, copy action, or hover text.
- Row actions should appear on hover only when they reduce clutter. Primary state-changing actions can also live in the inspector.

Do not implement the main table as unrelated cell buttons that visually fragment one row into many targets.

### Inspector

The right inspector shows details for the selected Skill, Agent, or Project deployment.

Common sections:

- Summary
- Source or deployment path
- Status
- Risk findings
- Actions
- Registry metadata

The inspector should avoid modal-first flows. Inline confirmation areas are preferred for destructive actions.

Inspector behavior:

- Inspector content scrolls independently from the app shell.
- Sections use stable key-value rows for Summary, Paths, Status, Metadata, and Actions.
- Paths, hashes, Skill IDs, and registry snippets use mono styling.
- Long descriptions wrap within the inspector and never push Controls off-screen without a scroll affordance.
- Path rows should expose copy and reveal affordances when the path points to a local file or directory.
- Empty inspector states should tell the user what selection or scan action is needed next.
- Switching the selected row may use a short fade or slide, but content must remain readable immediately.

### Badges

Badges are small, quiet, and consistent.

| State | Visual |
| --- | --- |
| Enabled | gray badge with success dot |
| Disabled | gray badge, muted text |
| Outdated | gray badge with warning dot |
| Drift | warning text or warning dot |
| Invalid | danger text or danger dot |
| Risk high | danger badge |
| Risk warning | warning badge |

Avoid full-color pill floods. Status should be scannable, not loud.

Badge implementation requirements:

- Badges use `surface_2` or `surface_3` backgrounds with semantic text or a small semantic dot.
- A badge must include text, not color alone.
- `Read-only` is informational unless paired with a blocking action.
- `Invalid`, `Missing`, and destructive risk states must be visually distinct from ordinary disabled state.
- Badge shape uses `xs` or `sm`; do not use pill-shaped large rounded badges.

### Inputs

Inputs use `surface_1`, `hairline`, and `focus` outline.

Path inputs should support:

- browse button
- reveal button
- validation message
- mono rendering for paths

Path input behavior:

- Opening a project should provide a native folder picker.
- Editing an Agent project Skill directory should provide a folder picker when possible.
- Existing local paths should provide Reveal in Finder.
- Invalid paths should show inline validation near the input.
- Drag and drop may be supported as a shortcut, but it must not be the only comfortable way to provide a path.
- Copying a path from Finder or terminal into the input should remain supported.
- Browse/reveal controls should use icon buttons with tooltips when icon assets are available; text buttons are acceptable in v0.1 if icon support is not yet in place.

### Empty States

Empty states are concise and action-led.

Examples:

- No Skills: `Install a local Skill or adopt existing Agent Skills.`
- No Recent Projects: `Open a project to scan project-level Skills.`
- No deployments: `Deploy a Managed Skill to this Agent project directory.`

Do not add explanatory marketing copy.

## Page Specifications

### Dashboard

Purpose: global overview.

Main content:

- Managed Skills count.
- Agent configuration summary.
- Recent Projects summary.
- Registry and lock health.
- Risk count last.

Layout:

- Compact summary strip at top.
- Recent Projects list below.
- Health panel in inspector or right rail.

Avoid large metric cards. Use rows and compact panels.

### Skill

Purpose: inspect and manage Agent Space Skill instances.

The Skill view is instance-first in the Agent Space model: one row represents one physical Skill directory that an Agent can read, or one legacy missing record that points to a vanished directory. Managed Inventory is summarized separately and does not become an Agent Space row unless an Agent config explicitly declares that directory as agent-readable.

Main table columns:

- Skill
- Agent
- Scope
- Status
- Source
- Managed
- Updated

Table requirements:

- One row equals one physical Agent Space Skill instance or one explicit legacy missing instance.
- Selecting any cell selects the whole row and updates the inspector.
- Status, Source, Managed, and Updated must be scan-friendly at compact row height.
- Long Skill names and paths should not increase row height.

Inspector:

- Real Skill directory.
- `SKILL.md` path.
- `SKILL.md.disabled` path.
- Agent and Scope.
- Writable or read-only state.
- Managed or unmanaged state.
- Metadata from the active toggle file.
- Content hash when available.
- Risk findings.
- Deployment references in Recent Projects when available.
- Actions: scan Agent Spaces, enable, disable, install local, import managed copy, deploy to project, uninstall managed copy.

Status values:

- Enabled
- Disabled
- Invalid
- Missing
- Read-only

Disable copy:

- `Disable changes SKILL.md to SKILL.md.disabled in the Agent Space. It does not delete the Skill directory.`

Uninstall managed copy:

- `Uninstall removes this managed copy from Managed Inventory. Agent Space copies are not deleted.`

### Agent

Purpose: configure Agent project skill directories.

Rows:

- Codex
- Claude Code
- Gemini CLI
- Custom Agents

Each row shows:

- Agent label.
- Project skill directories.
- Enabled state.
- Validation state.

Actions:

- edit path
- reset to default
- add custom Agent
- remove custom Agent

Do not show global Agent sync settings.

Path editing requirements:

- Agent path editing should not rely on manual typing alone.
- Built-in Agent rows should expose reset and validation clearly.
- Custom Agent creation should validate id, label, and project Skill directory before save.
- Directory values should render as mono text in row detail and inspector detail.

### Project

Purpose: manage one Project Scope.

Top area:

- project path
- refresh
- onboarding scan state
- Git ignore guidance

Agent grouping:

- Codex deployments
- Claude Code deployments
- Gemini deployments
- Custom Agent deployments

Deployment table columns:

- Skill
- Agent
- Toggle
- Outdated
- Drift
- Missing managed source
- Risk
- Path

Inspector actions:

- enable
- disable
- redeploy
- overwrite from managed
- promote to managed
- remove from project

Onboarding:

- Show discovered unmanaged project Skills.
- Offer Adopt all.
- Show partial success as `8 adopted, 2 conflicts`.
- Conflict actions: import as new, skip.

Project path requirements:

- Opening a project should support a native folder picker.
- Recent Projects should remain reachable from the sidebar even when the list grows.
- Project path display should render in mono and expose reveal/copy affordances.
- Refresh and scan status should be visible while work is running.

## Interaction Rules

### Deploy

Default deploy creates an enabled project deployment.

If target directory exists and is unmanaged:

- Show Deploy Conflict.
- Do not overwrite.
- Offer explanation and next steps: adopt, remove manually, or choose another Skill.

### Enable and Disable

Enable/disable only renames the toggle file.

Enable and disable operate on the selected Skill instance only. They do not mutate Managed Inventory, do not write an enablement flag to Registry, and do not affect same-name copies in other Agents or Projects.

Invalid toggle state:

- Both `SKILL.md` and `SKILL.md.disabled` exist.
- Both are missing.

Show invalid state as a blocking row status with repair guidance.

### Missing Managed Source

A deployment has missing managed source when the project copy still exists but the source Managed Skill no longer exists in Global Inventory.

Show this as `Missing managed source`.

Available actions:

- Promote to managed.
- Remove from project.

Do not auto-fix it.

### Redeploy

If drift exists, block default redeploy.

Show inline choices:

- Keep project copy.
- Overwrite from managed.
- Promote to managed.

Do not use a modal as the first interaction.

### Remove From Project

Remove deletes only the selected deployment directory.

If drift exists:

- GUI requires confirmation.
- Copy: `This project copy has local changes. Removing it deletes only this deployed Skill, not the Agent skill root.`

### Long Content Inspection

Long Skill content is expected. The GUI must treat long metadata as normal product data, not an edge case.

Requirements:

- Long Skill descriptions wrap in the inspector.
- Long path values do not resize table rows.
- Long command or risk snippets use mono styling and can be copied.
- When a selected Skill has more detail than fits, the user can scroll the inspector without losing the selected row state.

## Motion

Use minimal motion:

- row hover
- selection change
- inspector content fade or slide
- small progress indicator during scan/copy

Do not animate layout-heavy transitions. No bounce, elastic, or decorative motion.

Motion acceptance:

- Hover and selection feedback should feel immediate, with transitions in the 150 to 250ms range when animated.
- Running actions should show local progress near the initiating control or top status area.
- The UI must never wait for a decorative animation before allowing the next action.

## Accessibility

Requirements:

- Keyboard navigation for sidebar, tables, toolbar actions, and inspector actions.
- Visible focus state using `focus`.
- Do not rely on color alone for status. Pair color with label or icon/dot.
- Text contrast must remain high on dark surfaces.
- Destructive actions require clear copy and confirmation when data can be lost.
- All path browse, reveal, copy, refresh, enable, disable, and remove actions must be reachable without drag and drop.
- Scrollable panes must work with mouse wheel, trackpad, and keyboard focus traversal.

## egui Notes

egui does not have CSS. Implement tokens as Rust structs:

```rust
pub struct UiColors {
    pub canvas: Color32,
    pub surface_1: Color32,
    pub surface_2: Color32,
    pub surface_3: Color32,
    pub surface_4: Color32,
    pub hairline: Color32,
    pub hairline_strong: Color32,
    pub ink: Color32,
    pub ink_muted: Color32,
    pub ink_subtle: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub focus: Color32,
}
```

Use egui visuals to set:

- dark mode
- panel fill
- faint widget background
- selection fill
- hyperlink color as monochrome or muted gray, not lavender
- window rounding and widget rounding

Prefer custom row rendering for the main tables if standard widgets cannot reach the desired density.

Implementation notes for product polish:

- Use `egui::ScrollArea` or equivalent scroll containers for the main table, inspector, and any long sidebar list.
- Configure `eframe::NativeOptions.viewport` for macOS title bar integration instead of expecting egui dark visuals to recolor native chrome.
- Consider a reusable table row renderer before adding view-specific table behavior.
- Consider a reusable path field component for project opening, Agent directory editing, path reveal, copy, and validation.
- Keep action execution outside the egui render path. Long scans and file operations should continue to use background work and visible status updates.

## Productization Upgrade Spec

These items are required before the GUI can be considered productized beyond the current skeleton.

| Priority | Spec Item | Required Outcome |
| --- | --- | --- |
| P0 | Scroll and overflow support | Long Skill lists, long descriptions, and long paths remain usable in fixed-size windows. |
| P0 | Dark macOS title area | The launched macOS window has no white title strip above the dark workbench UI. |
| P1 | Path input workflow | Users can open projects and edit Agent directories with browse, reveal, validation, and copy support. |
| P1 | Workbench table renderer | Tables support whole-row selection, selected/hover states, compact density, status badges, and mono path fields. |
| P2 | Inspector polish | Inspector sections use key-value structure, independent scrolling, mono path rendering, and copy/reveal affordances. |
| P2 | Status badge system | Status values use consistent badge components with semantic dots or labels. |
| P2 | Minimal motion and progress | Row hover, selection change, inspector updates, and running actions have restrained stateful feedback. |
| P2 | Keyboard and non-drag access | Core workflows are available through keyboard/mouse controls without requiring drag and drop. |

Acceptance checklist:

- Launch the GUI on macOS and capture a screenshot showing the dark title area integrated with the app shell.
- Create or load enough Skill instances to require vertical scrolling in the main table.
- Select a Skill with long metadata and long paths; verify inspector scrolling, copy/reveal affordances, and visible action controls.
- Open a project using a folder picker, not only typed text.
- Edit an Agent project Skill directory using browse or reveal controls.
- Verify status badges are readable without relying on color alone.
- Verify keyboard focus can reach sidebar navigation, table rows, toolbar controls, inspector actions, and path controls.

## Bans

Do not use:

- lavender as brand color
- purple or blue gradients
- atmospheric glows
- glassmorphism
- hero sections
- large marketing metric cards
- repeated icon cards
- pill-shaped primary buttons
- nested cards
- global sync settings
- launcher or active directory language

## Locked Decisions

- v0.1 ships dark theme only.
- v0.1 uses platform system fonts and does not bundle custom fonts.
- `.gitignore` behavior is guidance only. The app does not edit `.gitignore` and does not force a commit-or-ignore recommendation.
