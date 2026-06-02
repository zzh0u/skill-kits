use crate::core::agent_space::SkillInstance;
use crate::gui::state::{
    skill_instance_source_label, skill_instance_status_label, GuiModel, InspectorSection,
    RenderRow, RenderableView,
};

pub fn view_name() -> &'static str {
    "Project"
}

pub fn renderable(model: &GuiModel) -> RenderableView {
    let selected_project = model.scope_project_path();
    let main_rows: Vec<_> = model
        .skill_instances
        .iter()
        .filter(|instance| {
            matches!(
                &instance.scope,
                crate::core::agent_space::SkillInstanceScope::Project { path, .. }
                    if selected_project.as_ref().map_or(true, |selected| selected == path)
            )
        })
        .map(|instance| RenderRow {
            id: instance.id.clone(),
            cells: vec![
                instance.name.clone(),
                model.agent_label(&instance.agent_id),
                skill_instance_status_label(instance).to_string(),
                skill_instance_source_label(model, instance),
                if instance.writable { "Yes" } else { "No" }.to_string(),
                instance.skill_dir.to_string(),
            ],
        })
        .collect();
    let empty_message = if main_rows.is_empty() {
        Some("No project Agent Space Skills in this scope. Scan Agent Spaces or open a project.")
    } else {
        None
    };

    RenderableView {
        view: model.active_view,
        title: view_name().to_string(),
        columns: vec![
            "Skill".to_string(),
            "Agent".to_string(),
            "Status".to_string(),
            "Source".to_string(),
            "Writable".to_string(),
            "Path".to_string(),
        ],
        main_rows,
        inspector_sections: inspector_sections(model),
        empty_message,
    }
}

fn inspector_sections(model: &GuiModel) -> Vec<InspectorSection> {
    if let Some(instance) = model.selected_skill_instance() {
        return vec![
            InspectorSection {
                title: "Project Skill".to_string(),
                lines: vec![
                    instance.name.clone(),
                    format!("Agent {}", model.agent_label(&instance.agent_id)),
                    format!("Status {}", skill_instance_status_label(instance)),
                    format!("Source {}", skill_instance_source_label(model, instance)),
                    format!("Writable {}", if instance.writable { "Yes" } else { "No" }),
                ],
            },
            InspectorSection {
                title: "Path".to_string(),
                lines: vec![
                    format!("Skill dir {}", instance.skill_dir),
                    format!("Enabled {}", instance.enabled_path),
                    format!("Disabled {}", instance.disabled_path),
                ],
            },
            InspectorSection {
                title: "Actions".to_string(),
                lines: action_lines(instance),
            },
        ];
    }

    if let Some(project) = model.selected_project_summary() {
        return vec![
            InspectorSection {
                title: "Project".to_string(),
                lines: vec![project.name.clone(), project.path.to_string()],
            },
            InspectorSection {
                title: "Native Skills".to_string(),
                lines: native_project_lines(model),
            },
            InspectorSection {
                title: "Git Ignore Guidance".to_string(),
                lines: vec!["Guidance only. Skill-kits does not edit .gitignore.".to_string()],
            },
        ];
    }

    vec![InspectorSection {
        title: "Empty".to_string(),
        lines: vec![
            "No Recent Project is selected.".to_string(),
            "Open a project from the Scope switcher before scanning or deploying.".to_string(),
        ],
    }]
}

fn native_project_lines(model: &GuiModel) -> Vec<String> {
    let Some(project) = model.selected_project_summary() else {
        return Vec::new();
    };
    let lines = model
        .skill_instances
        .iter()
        .filter(|instance| {
            matches!(
                &instance.scope,
                crate::core::agent_space::SkillInstanceScope::Project { path, .. }
                    if path == &project.path
            )
        })
        .map(|instance| {
            format!(
                "{}/{} - {}",
                instance.agent_id,
                instance.name,
                skill_instance_status_label(instance)
            )
        })
        .collect::<Vec<_>>();
    if lines.is_empty() {
        vec!["No native project Skill instances found.".to_string()]
    } else {
        lines
    }
}

fn action_lines(instance: &SkillInstance) -> Vec<String> {
    if matches!(
        instance.toggle_state,
        crate::core::registry::ToggleState::InvalidBothPresent
            | crate::core::registry::ToggleState::InvalidBothMissing
    ) {
        return vec![
            "Invalid toggle state blocks native actions.".to_string(),
            "Keep exactly one of SKILL.md or SKILL.md.disabled, then Refresh.".to_string(),
        ];
    }

    if !instance.writable {
        return vec!["Read-only project Skill instances cannot be toggled here.".to_string()];
    }

    vec![
        "Enable or Disable renames SKILL.md / SKILL.md.disabled for this instance only."
            .to_string(),
    ]
}
