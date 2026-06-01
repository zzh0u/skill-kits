use crate::core::{
    agent_space::{SkillInstance, SkillInstanceScope, SkillInstanceSourceKind},
    registry::ToggleState,
};
use crate::gui::state::{GuiModel, InspectorSection, RenderRow, RenderableView};

pub fn view_name() -> &'static str {
    "Skills"
}

pub fn renderable(model: &GuiModel) -> RenderableView {
    let main_rows = model
        .skill_instances
        .iter()
        .map(|instance| RenderRow {
            id: instance.id.clone(),
            cells: vec![
                instance.name.clone(),
                agent_label(model, instance),
                scope_label(&instance.scope),
                status_label(instance),
                source_label(model, instance),
                if instance.managed {
                    "Managed".to_string()
                } else {
                    "Unmanaged".to_string()
                },
                instance
                    .updated_at
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
            ],
        })
        .collect();

    RenderableView {
        view: model.active_view,
        title: view_name().to_string(),
        columns: vec![
            "Skill".to_string(),
            "Agent".to_string(),
            "Scope".to_string(),
            "Status".to_string(),
            "Source".to_string(),
            "Managed".to_string(),
            "Updated".to_string(),
        ],
        main_rows,
        inspector_sections: inspector_sections(model),
        empty_message: model
            .skill_instances
            .is_empty()
            .then_some("No Agent Space Skills found. Scan enabled Agent directories."),
    }
}

fn inspector_sections(model: &GuiModel) -> Vec<InspectorSection> {
    let Some(instance) = model
        .selected_skill_instance()
        .or_else(|| model.skill_instances.first())
    else {
        return vec![InspectorSection {
            title: "Empty".to_string(),
            lines: vec![
                "No Agent Space Skills found.".to_string(),
                "Scan enabled Agent directories or install a managed source.".to_string(),
            ],
        }];
    };

    let mut sections = vec![
        InspectorSection {
            title: "Summary".to_string(),
            lines: vec![
                instance.name.clone(),
                format!("Instance ID {}", instance.id),
                format!("Agent {}", agent_label(model, instance)),
                format!("Scope {}", scope_label(&instance.scope)),
                format!("Status {}", status_label(instance)),
                format!("Source {}", source_label(model, instance)),
                format!("Managed {}", if instance.managed { "Yes" } else { "No" }),
                format!("Writable {}", if instance.writable { "Yes" } else { "No" }),
            ],
        },
        InspectorSection {
            title: "Paths".to_string(),
            lines: vec![
                format!("Skill dir {}", instance.skill_dir),
                format!("Enabled {}", instance.enabled_path),
                format!("Disabled {}", instance.disabled_path),
            ],
        },
        InspectorSection {
            title: "Metadata".to_string(),
            lines: metadata_lines(instance),
        },
        InspectorSection {
            title: "Registry Metadata".to_string(),
            lines: registry_metadata_lines(instance),
        },
        InspectorSection {
            title: "Actions".to_string(),
            lines: vec![
                "Scan Agent Spaces refreshes the Agent-visible Skill read model.".to_string(),
                "Install local copies a local Skill into Managed Inventory.".to_string(),
                "Deploy to Project copies a managed Skill into a project Agent Space.".to_string(),
            ],
        },
    ];

    sections.retain(|section| !section.lines.is_empty());
    sections
}

fn metadata_lines(instance: &SkillInstance) -> Vec<String> {
    let Some(metadata) = &instance.metadata else {
        return Vec::new();
    };

    let mut lines = Vec::new();
    if let Some(title) = &metadata.title {
        lines.push(format!("Title {title}"));
    }
    if let Some(description) = &metadata.description {
        lines.push(format!("Description {description}"));
    }
    lines
}

fn registry_metadata_lines(instance: &SkillInstance) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(stable_id) = &instance.stable_id {
        lines.push(format!("Stable ID {stable_id}"));
    }
    if let Some(content_hash) = &instance.content_hash {
        lines.push(format!("Hash {content_hash}"));
    }
    if let Some(updated_at) = &instance.updated_at {
        lines.push(format!("Updated {updated_at}"));
    }
    lines
}

fn agent_label(model: &GuiModel, instance: &SkillInstance) -> String {
    model
        .agents
        .iter()
        .find(|agent| agent.id == instance.agent_id)
        .map(|agent| agent.label.clone())
        .unwrap_or_else(|| instance.agent_id.to_string())
}

fn scope_label(scope: &SkillInstanceScope) -> String {
    match scope {
        SkillInstanceScope::Global => "Global".to_string(),
        SkillInstanceScope::Project { name, .. } => format!("Project / {name}"),
    }
}

fn status_label(instance: &SkillInstance) -> String {
    if !instance.writable
        && matches!(
            instance.toggle_state,
            ToggleState::Enabled | ToggleState::Disabled
        )
    {
        return "Read-only".to_string();
    }
    match instance.toggle_state {
        ToggleState::Enabled => "Enabled".to_string(),
        ToggleState::Disabled => "Disabled".to_string(),
        ToggleState::InvalidBothPresent => "Invalid".to_string(),
        ToggleState::InvalidBothMissing => "Missing".to_string(),
    }
}

fn source_label(model: &GuiModel, instance: &SkillInstance) -> String {
    match &instance.source_kind {
        SkillInstanceSourceKind::AgentSpace => {
            format!("{} global", agent_label(model, instance))
        }
        SkillInstanceSourceKind::ProjectDeployment => "Project".to_string(),
        SkillInstanceSourceKind::PluginCache => "Plugin cache".to_string(),
        SkillInstanceSourceKind::Vendor => "Vendor".to_string(),
        SkillInstanceSourceKind::ManagedInventory => "Managed inventory".to_string(),
    }
}
