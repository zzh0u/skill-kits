use crate::core::{
    agents::configured_global_skill_dirs_for,
    config::read_config,
    error::{Result, SkillKitsError},
    fs::{atomic_write_toml, safe_read_to_string},
    ids::{AgentId, SkillId},
    lock::StateLock,
    paths::{ensure_app_dirs, AppPaths},
    registry::{SkillMetadata, ToggleState},
    skills::{disabled_skill_markdown_path, load_skill_metadata_from_file, skill_markdown_path},
};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;

pub const AGENT_SPACE_PLUGIN_DEPTH: usize = 6;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SkillInstanceScope {
    Global,
    Project { name: String, path: Utf8PathBuf },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum SkillInstanceSourceKind {
    AgentSpace,
    ProjectAgentSpace,
    PluginCache,
    Vendor,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Clone, Debug)]
pub struct SkillInstanceRequest<'a> {
    pub app_paths: &'a AppPaths,
    pub home_dir: &'a Utf8Path,
    pub instance_id: &'a str,
}

#[derive(Clone, Debug)]
pub struct SkillInstanceQueryRequest<'a> {
    pub app_paths: &'a AppPaths,
    pub home_dir: &'a Utf8Path,
    pub query: &'a str,
}

#[derive(Clone, Debug)]
pub struct ProjectSkillInstanceRequest<'a> {
    pub app_paths: &'a AppPaths,
    pub home_dir: &'a Utf8Path,
    pub project_path: &'a Utf8Path,
    pub agent_id: &'a AgentId,
    pub skill_query: &'a str,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillInstanceIndex {
    pub version: u32,
    pub last_scanned_at: String,
    #[serde(default)]
    pub instances: Vec<SkillInstance>,
}

impl Default for SkillInstanceIndex {
    fn default() -> Self {
        Self {
            version: 1,
            last_scanned_at: String::new(),
            instances: Vec::new(),
        }
    }
}

pub fn refresh_skill_instance_index(
    app_paths: &AppPaths,
    home_dir: &Utf8Path,
) -> Result<SkillInstanceIndex> {
    let index = SkillInstanceIndex {
        version: 1,
        last_scanned_at: now_string(),
        instances: scan_agent_spaces(app_paths, home_dir)?,
    };
    write_skill_instance_index(app_paths, &index)?;
    Ok(index)
}

pub fn read_skill_instance_index(app_paths: &AppPaths) -> Result<SkillInstanceIndex> {
    ensure_app_dirs(app_paths)?;
    let index = read_skill_instance_index_unlocked(app_paths)?;
    if index_has_stale_paths(&index) {
        if let Some(home_dir) = home_dir_for_index_rescan(app_paths, &index) {
            return refresh_skill_instance_index(app_paths, &home_dir);
        }
    }
    Ok(index)
}

pub fn write_skill_instance_index(app_paths: &AppPaths, index: &SkillInstanceIndex) -> Result<()> {
    let _lock = StateLock::acquire(app_paths)?;
    ensure_app_dirs(app_paths)?;
    atomic_write_toml(&app_paths.skill_instance_index_file, index)
}

fn read_skill_instance_index_unlocked(app_paths: &AppPaths) -> Result<SkillInstanceIndex> {
    if !app_paths.skill_instance_index_file.exists() {
        return Ok(SkillInstanceIndex::default());
    }

    let contents = safe_read_to_string(&app_paths.skill_instance_index_file)?;
    toml::from_str(&contents).map_err(|source| SkillKitsError::RegistryParse {
        path: app_paths.skill_instance_index_file.clone(),
        source,
    })
}

fn index_has_stale_paths(index: &SkillInstanceIndex) -> bool {
    index
        .instances
        .iter()
        .any(|instance| !has_toggle_file(&instance.skill_dir))
}

fn home_dir_for_index_rescan(
    app_paths: &AppPaths,
    index: &SkillInstanceIndex,
) -> Option<Utf8PathBuf> {
    for instance in &index.instances {
        if !matches!(instance.scope, SkillInstanceScope::Global) {
            continue;
        }
        let dirs = configured_global_skill_dirs_for(app_paths, &instance.agent_id).ok()?;
        for root in dirs {
            if let Some(home_dir) = infer_home_dir_from_tilde_root(&instance.skill_dir, &root) {
                return Some(home_dir);
            }
        }
    }

    dirs::home_dir().and_then(|path| Utf8PathBuf::from_path_buf(path).ok())
}

fn infer_home_dir_from_tilde_root(skill_dir: &Utf8Path, root: &Utf8Path) -> Option<Utf8PathBuf> {
    let rest = root.as_str().strip_prefix("~/")?;
    let needle = format!("/{rest}/");
    let index = skill_dir.as_str().find(&needle)?;
    let home = &skill_dir.as_str()[..index];
    if home.is_empty() {
        return None;
    }
    Some(Utf8PathBuf::from(home))
}

pub fn scan_agent_spaces(app_paths: &AppPaths, home_dir: &Utf8Path) -> Result<Vec<SkillInstance>> {
    let config = read_config(app_paths)?;

    let mut instances = Vec::new();
    let mut seen = HashSet::new();

    for agent in config.agents.iter().filter(|agent| agent.enabled) {
        let global_skill_dirs = configured_global_skill_dirs_for(app_paths, &agent.id)?;
        for root in &global_skill_dirs {
            let root = expand_home(root, home_dir);
            let source_kind = source_kind_for_root(&root);
            let root_instances = if is_recursive_source(&source_kind) {
                scan_recursive_root(
                    &root,
                    agent.id.clone(),
                    SkillInstanceScope::Global,
                    source_kind,
                )?
            } else {
                scan_immediate_root(
                    &root,
                    agent.id.clone(),
                    SkillInstanceScope::Global,
                    source_kind,
                )?
            };
            push_unique(&mut instances, &mut seen, root_instances);
        }

        for recent in &config.recent_projects {
            for project_dir in &agent.project_skill_dirs {
                let root = recent.path.join(project_dir);
                let root_instances = scan_immediate_root(
                    &root,
                    agent.id.clone(),
                    SkillInstanceScope::Project {
                        name: recent.name.clone(),
                        path: recent.path.clone(),
                    },
                    SkillInstanceSourceKind::ProjectAgentSpace,
                )?;
                push_unique(&mut instances, &mut seen, root_instances);
            }
        }
    }

    instances.sort_by(|left, right| {
        (
            left.agent_id.as_str(),
            scope_sort_key(&left.scope),
            left.skill_dir.as_str(),
        )
            .cmp(&(
                right.agent_id.as_str(),
                scope_sort_key(&right.scope),
                right.skill_dir.as_str(),
            ))
    });
    Ok(instances)
}

pub fn enable_skill_instance(request: SkillInstanceRequest<'_>) -> Result<SkillInstance> {
    let instance = find_skill_instance(&request)?;
    ensure_instance_can_toggle(&instance)?;
    if instance.disabled_path.exists() {
        fs::rename(&instance.disabled_path, &instance.enabled_path)?;
    }
    let index = refresh_skill_instance_index(request.app_paths, request.home_dir)?;
    index
        .instances
        .into_iter()
        .find(|instance| instance.id == request.instance_id)
        .ok_or_else(|| SkillKitsError::SkillNotFound {
            query: request.instance_id.to_string(),
        })
}

pub fn disable_skill_instance(request: SkillInstanceRequest<'_>) -> Result<SkillInstance> {
    let instance = find_skill_instance(&request)?;
    ensure_instance_can_toggle(&instance)?;
    if instance.enabled_path.exists() {
        fs::rename(&instance.enabled_path, &instance.disabled_path)?;
    }
    let index = refresh_skill_instance_index(request.app_paths, request.home_dir)?;
    index
        .instances
        .into_iter()
        .find(|instance| instance.id == request.instance_id)
        .ok_or_else(|| SkillKitsError::SkillNotFound {
            query: request.instance_id.to_string(),
        })
}

pub fn enable_skill_instance_by_query(
    request: SkillInstanceQueryRequest<'_>,
) -> Result<SkillInstance> {
    let instance = find_skill_instance_by_query(
        scan_agent_spaces(request.app_paths, request.home_dir)?,
        request.query,
    )?;
    enable_skill_instance(SkillInstanceRequest {
        app_paths: request.app_paths,
        home_dir: request.home_dir,
        instance_id: &instance.id,
    })
}

pub fn disable_skill_instance_by_query(
    request: SkillInstanceQueryRequest<'_>,
) -> Result<SkillInstance> {
    let instance = find_skill_instance_by_query(
        scan_agent_spaces(request.app_paths, request.home_dir)?,
        request.query,
    )?;
    disable_skill_instance(SkillInstanceRequest {
        app_paths: request.app_paths,
        home_dir: request.home_dir,
        instance_id: &instance.id,
    })
}

pub fn project_skill_instances(
    app_paths: &AppPaths,
    home_dir: &Utf8Path,
    project_path: &Utf8Path,
) -> Result<Vec<SkillInstance>> {
    let mut instances = scan_agent_spaces(app_paths, home_dir)?
        .into_iter()
        .filter(|instance| matches_project_scope(&instance.scope, project_path))
        .collect::<Vec<_>>();
    instances.sort_by(|left, right| {
        (
            left.agent_id.as_str(),
            left.skill_dir.file_name().unwrap_or(left.name.as_str()),
            left.name.as_str(),
        )
            .cmp(&(
                right.agent_id.as_str(),
                right.skill_dir.file_name().unwrap_or(right.name.as_str()),
                right.name.as_str(),
            ))
    });
    Ok(instances)
}

pub fn project_skill_instance_status(
    request: ProjectSkillInstanceRequest<'_>,
) -> Result<SkillInstance> {
    find_project_skill_instance(&request)
}

pub fn enable_project_skill_instance(
    request: ProjectSkillInstanceRequest<'_>,
) -> Result<SkillInstance> {
    let instance = find_project_skill_instance(&request)?;
    enable_skill_instance(SkillInstanceRequest {
        app_paths: request.app_paths,
        home_dir: request.home_dir,
        instance_id: &instance.id,
    })
}

pub fn disable_project_skill_instance(
    request: ProjectSkillInstanceRequest<'_>,
) -> Result<SkillInstance> {
    let instance = find_project_skill_instance(&request)?;
    disable_skill_instance(SkillInstanceRequest {
        app_paths: request.app_paths,
        home_dir: request.home_dir,
        instance_id: &instance.id,
    })
}

fn find_skill_instance(request: &SkillInstanceRequest<'_>) -> Result<SkillInstance> {
    scan_agent_spaces(request.app_paths, request.home_dir)?
        .into_iter()
        .find(|instance| instance.id == request.instance_id)
        .ok_or_else(|| SkillKitsError::SkillNotFound {
            query: request.instance_id.to_string(),
        })
}

fn find_project_skill_instance(request: &ProjectSkillInstanceRequest<'_>) -> Result<SkillInstance> {
    let instances =
        project_skill_instances(request.app_paths, request.home_dir, request.project_path)?
            .into_iter()
            .filter(|instance| instance.agent_id == *request.agent_id)
            .collect::<Vec<_>>();
    find_skill_instance_by_query(instances, request.skill_query)
}

fn find_skill_instance_by_query(
    instances: Vec<SkillInstance>,
    query: &str,
) -> Result<SkillInstance> {
    let mut matches = instances
        .into_iter()
        .filter(|instance| instance_matches_query(instance, query))
        .collect::<Vec<_>>();
    if matches.len() == 1 {
        return Ok(matches.remove(0));
    }
    if matches.is_empty() {
        return Err(SkillKitsError::SkillNotFound {
            query: query.to_string(),
        });
    }
    matches.sort_by(|left, right| left.id.cmp(&right.id));
    Err(SkillKitsError::AmbiguousSkill {
        query: query.to_string(),
        matches: matches
            .into_iter()
            .map(|instance| SkillId::new(instance.id))
            .collect(),
    })
}

fn instance_matches_query(instance: &SkillInstance, query: &str) -> bool {
    instance.id == query
        || instance.name == query
        || instance
            .skill_dir
            .file_name()
            .is_some_and(|name| name == query)
}

fn matches_project_scope(scope: &SkillInstanceScope, project_path: &Utf8Path) -> bool {
    let SkillInstanceScope::Project { path, .. } = scope else {
        return false;
    };
    paths_refer_to_same_location(path, project_path)
}

fn paths_refer_to_same_location(left: &Utf8Path, right: &Utf8Path) -> bool {
    if left == right {
        return true;
    }
    let left = fs::canonicalize(left.as_std_path())
        .ok()
        .and_then(|path| Utf8PathBuf::from_path_buf(path).ok());
    let right = fs::canonicalize(right.as_std_path())
        .ok()
        .and_then(|path| Utf8PathBuf::from_path_buf(path).ok());
    left.is_some() && left == right
}

fn ensure_instance_can_toggle(instance: &SkillInstance) -> Result<()> {
    let source_allows_toggle = matches!(
        instance.source_kind,
        SkillInstanceSourceKind::AgentSpace | SkillInstanceSourceKind::ProjectAgentSpace
    );
    let valid_toggle = matches!(
        instance.toggle_state,
        ToggleState::Enabled | ToggleState::Disabled
    );
    if instance.writable && source_allows_toggle && valid_toggle {
        return Ok(());
    }
    Err(SkillKitsError::InvalidToggleState {
        path: instance.skill_dir.clone(),
    })
}

fn scan_immediate_root(
    root: &Utf8Path,
    agent_id: AgentId,
    scope: SkillInstanceScope,
    source_kind: SkillInstanceSourceKind,
) -> Result<Vec<SkillInstance>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();
    for entry in fs::read_dir(root.as_std_path())? {
        let entry = entry?;
        let skill_dir = utf8_path(entry.path())?;
        if !skill_dir.is_dir() {
            continue;
        }
        if has_toggle_file(&skill_dir) {
            instances.push(build_instance(
                skill_dir,
                agent_id.clone(),
                scope.clone(),
                source_kind.clone(),
            )?);
        }
    }
    Ok(instances)
}

fn scan_recursive_root(
    root: &Utf8Path,
    agent_id: AgentId,
    scope: SkillInstanceScope,
    source_kind: SkillInstanceSourceKind,
) -> Result<Vec<SkillInstance>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();
    let mut walker = WalkDir::new(root)
        .follow_links(false)
        .max_depth(AGENT_SPACE_PLUGIN_DEPTH)
        .into_iter()
        .filter_entry(|entry| {
            if entry.depth() == 0 {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            !is_noisy_dir(&name)
        });

    while let Some(entry) = walker.next() {
        let entry = entry.map_err(|source| std::io::Error::other(source.to_string()))?;
        if !entry.file_type().is_dir() || entry.depth() == 0 {
            continue;
        }
        let skill_dir = utf8_path(entry.path().to_path_buf())?;
        if has_toggle_file(&skill_dir) {
            walker.skip_current_dir();
            instances.push(build_instance(
                skill_dir,
                agent_id.clone(),
                scope.clone(),
                source_kind.clone(),
            )?);
        }
    }
    Ok(instances)
}

fn build_instance(
    skill_dir: Utf8PathBuf,
    agent_id: AgentId,
    scope: SkillInstanceScope,
    source_kind: SkillInstanceSourceKind,
) -> Result<SkillInstance> {
    let enabled_path = skill_markdown_path(&skill_dir);
    let disabled_path = disabled_skill_markdown_path(&skill_dir);
    let toggle_state = toggle_state(&skill_dir);
    let toggle_file = match toggle_state {
        ToggleState::Enabled => Some(enabled_path.as_path()),
        ToggleState::Disabled => Some(disabled_path.as_path()),
        ToggleState::InvalidBothPresent | ToggleState::InvalidBothMissing => None,
    };
    let metadata = toggle_file
        .map(load_skill_metadata_from_file)
        .transpose()?
        .flatten();
    let name = if matches!(toggle_state, ToggleState::InvalidBothPresent) {
        dir_name(&skill_dir)
    } else {
        metadata
            .as_ref()
            .and_then(|metadata| metadata.title.clone())
            .unwrap_or_else(|| dir_name(&skill_dir))
    };
    let content_hash = if matches!(toggle_state, ToggleState::Enabled | ToggleState::Disabled) {
        Some(hash_agent_skill_dir(&skill_dir)?)
    } else {
        None
    };
    let updated_at = toggle_file.and_then(file_updated_at);
    let read_only_source = matches!(
        source_kind,
        SkillInstanceSourceKind::PluginCache | SkillInstanceSourceKind::Vendor
    );
    let valid_toggle = matches!(toggle_state, ToggleState::Enabled | ToggleState::Disabled);
    let writable = valid_toggle && !read_only_source && is_writable_dir(&skill_dir);

    Ok(SkillInstance {
        id: instance_id(&agent_id, &scope, &skill_dir),
        name,
        agent_id,
        scope,
        skill_dir: skill_dir.clone(),
        enabled_path,
        disabled_path,
        toggle_state,
        source_kind,
        writable,
        metadata,
        content_hash,
        updated_at,
    })
}

fn hash_agent_skill_dir(skill_dir: &Utf8Path) -> Result<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(skill_dir).follow_links(true) {
        let entry = entry.map_err(|source| std::io::Error::other(source.to_string()))?;
        if entry.file_type().is_dir() {
            continue;
        }
        let path = utf8_path(entry.path().to_path_buf())?;
        let rel = path.strip_prefix(skill_dir).unwrap();
        if should_ignore_hash_path(rel) {
            continue;
        }
        files.push(path);
    }

    files.sort();
    let mut hasher = Sha256::new();
    for file in files {
        let rel = file.strip_prefix(skill_dir).unwrap();
        let normalized = normalize_hash_path(rel);
        hasher.update(normalized.as_bytes());
        hasher.update([0]);
        let bytes = fs::read(file.as_std_path())?;
        hasher.update(bytes.len().to_le_bytes());
        hasher.update([0]);
        hasher.update(bytes);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn instance_id(agent_id: &AgentId, scope: &SkillInstanceScope, skill_dir: &Utf8Path) -> String {
    let canonical = fs::canonicalize(skill_dir.as_std_path())
        .ok()
        .and_then(|path| Utf8PathBuf::from_path_buf(path).ok())
        .unwrap_or_else(|| skill_dir.to_path_buf());
    let mut hasher = Sha256::new();
    hasher.update(agent_id.as_str().as_bytes());
    hasher.update([0]);
    hasher.update(scope_key(scope).as_bytes());
    hasher.update([0]);
    hasher.update(canonical.as_str().as_bytes());
    hex::encode(hasher.finalize())
}

fn push_unique(
    instances: &mut Vec<SkillInstance>,
    seen: &mut HashSet<String>,
    candidates: Vec<SkillInstance>,
) {
    for instance in candidates {
        if seen.insert(instance.id.clone()) {
            instances.push(instance);
        }
    }
}

fn expand_home(path: &Utf8Path, home_dir: &Utf8Path) -> Utf8PathBuf {
    if path == Utf8Path::new("~") {
        return home_dir.to_path_buf();
    }
    if let Some(rest) = path.as_str().strip_prefix("~/") {
        return home_dir.join(rest);
    }
    path.to_path_buf()
}

fn source_kind_for_root(root: &Utf8Path) -> SkillInstanceSourceKind {
    let components = root.components().map(|component| component.as_str());
    let mut previous = "";
    let mut source_kind = SkillInstanceSourceKind::AgentSpace;
    for component in components {
        if previous == "plugins" && component == "cache" {
            source_kind = SkillInstanceSourceKind::PluginCache;
            break;
        }
        if previous == ".codex" && component == "vendor_imports" {
            source_kind = SkillInstanceSourceKind::Vendor;
            break;
        }
        previous = component;
    }

    if matches!(source_kind, SkillInstanceSourceKind::PluginCache) {
        SkillInstanceSourceKind::PluginCache
    } else if matches!(source_kind, SkillInstanceSourceKind::Vendor) {
        SkillInstanceSourceKind::Vendor
    } else {
        SkillInstanceSourceKind::AgentSpace
    }
}

fn is_recursive_source(source_kind: &SkillInstanceSourceKind) -> bool {
    matches!(
        source_kind,
        SkillInstanceSourceKind::PluginCache | SkillInstanceSourceKind::Vendor
    )
}

fn has_toggle_file(skill_dir: &Utf8Path) -> bool {
    skill_markdown_path(skill_dir).exists() || disabled_skill_markdown_path(skill_dir).exists()
}

fn toggle_state(dir: &Utf8Path) -> ToggleState {
    let enabled = skill_markdown_path(dir).exists();
    let disabled = disabled_skill_markdown_path(dir).exists();
    match (enabled, disabled) {
        (true, false) => ToggleState::Enabled,
        (false, true) => ToggleState::Disabled,
        (true, true) => ToggleState::InvalidBothPresent,
        (false, false) => ToggleState::InvalidBothMissing,
    }
}

fn normalize_hash_path(path: &Utf8Path) -> String {
    if path.file_name() == Some("SKILL.md.disabled") {
        path.with_file_name("SKILL.md").to_string()
    } else {
        path.to_string()
    }
}

fn should_ignore_hash_path(path: &Utf8Path) -> bool {
    let file_name = path.file_name().unwrap_or_default();
    file_name == ".DS_Store"
        || (file_name.starts_with('.') && file_name.ends_with(".tmp"))
        || file_name.starts_with(".#")
        || file_name.ends_with('~')
        || file_name.ends_with(".swp")
        || file_name.ends_with(".swo")
        || file_name.starts_with(".skill-kits")
}

fn is_noisy_dir(name: &str) -> bool {
    (name.starts_with('.') && name != ".curated")
        || matches!(
            name,
            "node_modules"
                | "target"
                | "__pycache__"
                | "vendor"
                | "dist"
                | "build"
                | "venv"
                | ".venv"
        )
}

fn dir_name(path: &Utf8Path) -> String {
    path.file_name()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path.to_string())
}

fn is_writable_dir(path: &Utf8Path) -> bool {
    fs::metadata(path.as_std_path())
        .map(|metadata| !metadata.permissions().readonly())
        .unwrap_or(false)
}

fn file_updated_at(path: &Utf8Path) -> Option<String> {
    let modified = fs::metadata(path.as_std_path()).ok()?.modified().ok()?;
    let secs = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(secs.to_string())
}

fn now_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}

fn scope_key(scope: &SkillInstanceScope) -> String {
    match scope {
        SkillInstanceScope::Global => "global".to_string(),
        SkillInstanceScope::Project { path, .. } => format!("project:{path}"),
    }
}

fn scope_sort_key(scope: &SkillInstanceScope) -> String {
    match scope {
        SkillInstanceScope::Global => "0:global".to_string(),
        SkillInstanceScope::Project { name, path } => format!("1:{name}:{path}"),
    }
}

fn utf8_path(path: std::path::PathBuf) -> Result<Utf8PathBuf> {
    Utf8PathBuf::from_path_buf(path).map_err(|path| SkillKitsError::InvalidSkillDir {
        path: Utf8PathBuf::from(path.to_string_lossy().to_string()),
        reason: "path is not UTF-8".to_string(),
    })
}
