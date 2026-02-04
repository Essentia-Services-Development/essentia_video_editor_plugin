//! Project Manager for Essentia Video Editor Plugin
//! GAP-220-B-008: Project Save/Load System
//!
//! Features: Project save/load, autosave, version control,
//! recovery, project templates, and recent files.

use crate::{
    errors::{VideoEditorError, VideoEditorResult},
    types::Timestamp,
};

/// Unique identifier for a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId(u64);

impl ProjectId {
    /// Creates a new project ID.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner ID value.
    #[must_use]
    pub const fn inner(&self) -> u64 {
        self.0
    }
}

/// Project state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProjectState {
    /// New project, never saved.
    #[default]
    New,
    /// Project has been saved and is clean.
    Saved,
    /// Project has unsaved changes.
    Modified,
    /// Project is being saved.
    Saving,
    /// Project is being loaded.
    Loading,
    /// Project is read-only.
    ReadOnly,
    /// Project has errors.
    Error,
}

/// Project file format version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectVersion {
    /// Major version.
    pub major: u16,
    /// Minor version.
    pub minor: u16,
    /// Patch version.
    pub patch: u16,
}

impl ProjectVersion {
    /// Current project format version.
    pub const CURRENT: Self = Self { major: 1, minor: 0, patch: 0 };

    /// Creates a new version.
    #[must_use]
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }

    /// Checks if this version is compatible with another.
    #[must_use]
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Major version must match, minor can be higher
        self.major == other.major && self.minor >= other.minor
    }
}

impl Default for ProjectVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

/// Project metadata.
#[derive(Debug, Clone, Default)]
pub struct ProjectMetadata {
    /// Project name.
    pub name:        String,
    /// Project description.
    pub description: String,
    /// Author name.
    pub author:      String,
    /// Copyright notice.
    pub copyright:   String,
    /// Project tags.
    pub tags:        Vec<String>,
    /// Custom metadata fields.
    pub custom:      Vec<(String, String)>,
    /// Creation timestamp.
    pub created_at:  Timestamp,
    /// Last modified timestamp.
    pub modified_at: Timestamp,
    /// Project version.
    pub version:     ProjectVersion,
    /// Software version that created this project.
    pub app_version: String,
}

impl ProjectMetadata {
    /// Creates new metadata with default values.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let now = Timestamp::now();
        Self {
            name:        name.into(),
            description: String::new(),
            author:      String::new(),
            copyright:   String::new(),
            tags:        Vec::new(),
            custom:      Vec::new(),
            created_at:  now,
            modified_at: now,
            version:     ProjectVersion::CURRENT,
            app_version: "1.0.0".into(),
        }
    }

    /// Updates the modification timestamp.
    pub fn touch(&mut self) {
        self.modified_at = Timestamp::now();
    }
}

/// Project settings.
#[derive(Debug, Clone)]
pub struct ProjectSettings {
    /// Timeline resolution (width).
    pub timeline_width:    u32,
    /// Timeline resolution (height).
    pub timeline_height:   u32,
    /// Frame rate numerator.
    pub frame_rate_num:    u32,
    /// Frame rate denominator.
    pub frame_rate_den:    u32,
    /// Sample rate for audio.
    pub sample_rate:       u32,
    /// Color space.
    pub color_space:       String,
    /// Pixel aspect ratio.
    pub pixel_aspect:      f64,
    /// Working color depth (8, 10, 16, 32).
    pub color_depth:       u8,
    /// Preview quality (0.0 to 1.0).
    pub preview_quality:   f32,
    /// Proxy generation enabled.
    pub use_proxies:       bool,
    /// Auto-save interval in seconds (0 = disabled).
    pub autosave_interval: u32,
    /// Maximum undo history.
    pub max_undo_history:  u32,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            timeline_width:    1920,
            timeline_height:   1080,
            frame_rate_num:    30,
            frame_rate_den:    1,
            sample_rate:       48000,
            color_space:       "sRGB".into(),
            pixel_aspect:      1.0,
            color_depth:       8,
            preview_quality:   0.5,
            use_proxies:       true,
            autosave_interval: 300, // 5 minutes
            max_undo_history:  100,
        }
    }
}

impl ProjectSettings {
    /// Returns the frame rate as f64.
    #[must_use]
    pub fn frame_rate(&self) -> f64 {
        if self.frame_rate_den == 0 {
            0.0
        } else {
            self.frame_rate_num as f64 / self.frame_rate_den as f64
        }
    }

    /// Creates 4K settings.
    #[must_use]
    pub fn uhd_4k() -> Self {
        Self {
            timeline_width: 3840,
            timeline_height: 2160,
            color_depth: 10,
            ..Default::default()
        }
    }

    /// Creates film settings (24fps).
    #[must_use]
    pub fn film() -> Self {
        Self { frame_rate_num: 24, frame_rate_den: 1, ..Default::default() }
    }

    /// Creates NTSC settings (29.97fps).
    #[must_use]
    pub fn ntsc() -> Self {
        Self { frame_rate_num: 30000, frame_rate_den: 1001, ..Default::default() }
    }
}

/// Autosave information.
#[derive(Debug, Clone)]
pub struct AutosaveInfo {
    /// Path to autosave file.
    pub path:        String,
    /// Timestamp of autosave.
    pub timestamp:   Timestamp,
    /// Whether this is a recovery file.
    pub is_recovery: bool,
}

/// A video editing project.
#[derive(Debug)]
pub struct Project {
    /// Project identifier.
    id:              ProjectId,
    /// File path (None if never saved).
    path:            Option<String>,
    /// Project metadata.
    metadata:        ProjectMetadata,
    /// Project settings.
    settings:        ProjectSettings,
    /// Current state.
    state:           ProjectState,
    /// Undo stack (serialized states).
    undo_stack:      Vec<Vec<u8>>,
    /// Redo stack (serialized states).
    redo_stack:      Vec<Vec<u8>>,
    /// Current undo index.
    undo_index:      usize,
    /// Last autosave info.
    last_autosave:   Option<AutosaveInfo>,
    /// Asset paths referenced by project.
    asset_paths:     Vec<String>,
    /// Linked projects (for team workflows).
    linked_projects: Vec<String>,
}

impl Project {
    /// Creates a new project.
    #[must_use]
    pub fn new(id: ProjectId, name: impl Into<String>) -> Self {
        Self {
            id,
            path: None,
            metadata: ProjectMetadata::new(name),
            settings: ProjectSettings::default(),
            state: ProjectState::New,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            undo_index: 0,
            last_autosave: None,
            asset_paths: Vec::new(),
            linked_projects: Vec::new(),
        }
    }

    /// Returns the project ID.
    #[must_use]
    pub const fn id(&self) -> ProjectId {
        self.id
    }

    /// Returns the file path.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    /// Sets the file path.
    pub fn set_path(&mut self, path: impl Into<String>) {
        self.path = Some(path.into());
    }

    /// Returns the project metadata.
    #[must_use]
    pub fn metadata(&self) -> &ProjectMetadata {
        &self.metadata
    }

    /// Returns mutable metadata.
    pub fn metadata_mut(&mut self) -> &mut ProjectMetadata {
        &mut self.metadata
    }

    /// Returns the project settings.
    #[must_use]
    pub fn settings(&self) -> &ProjectSettings {
        &self.settings
    }

    /// Returns mutable settings.
    pub fn settings_mut(&mut self) -> &mut ProjectSettings {
        &mut self.settings
    }

    /// Returns the current state.
    #[must_use]
    pub const fn state(&self) -> ProjectState {
        self.state
    }

    /// Returns whether the project has unsaved changes.
    #[must_use]
    pub fn has_unsaved_changes(&self) -> bool {
        matches!(self.state, ProjectState::Modified | ProjectState::New)
    }

    /// Marks the project as modified.
    pub fn mark_modified(&mut self) {
        if !matches!(self.state, ProjectState::ReadOnly | ProjectState::Error) {
            self.state = ProjectState::Modified;
            self.metadata.touch();
        }
    }

    /// Marks the project as saved.
    pub fn mark_saved(&mut self) {
        self.state = ProjectState::Saved;
    }

    /// Returns whether undo is available.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.undo_index > 0
    }

    /// Returns whether redo is available.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.undo_index < self.undo_stack.len()
    }

    /// Pushes a state for undo.
    pub fn push_undo_state(&mut self, state: Vec<u8>) {
        // Clear redo stack when new action is performed
        if self.undo_index < self.undo_stack.len() {
            self.undo_stack.truncate(self.undo_index);
        }
        self.redo_stack.clear();

        // Add new state
        self.undo_stack.push(state);
        self.undo_index = self.undo_stack.len();

        // Limit undo history
        let max = self.settings.max_undo_history as usize;
        if self.undo_stack.len() > max {
            let remove = self.undo_stack.len() - max;
            self.undo_stack.drain(0..remove);
            self.undo_index = self.undo_index.saturating_sub(remove);
        }

        self.mark_modified();
    }

    /// Pops state for undo.
    pub fn pop_undo_state(&mut self) -> Option<Vec<u8>> {
        if self.undo_index > 0 {
            self.undo_index -= 1;
            // Save current state to redo stack
            if let Some(state) = self.undo_stack.get(self.undo_index).cloned() {
                self.redo_stack.push(state.clone());
                return Some(state);
            }
        }
        None
    }

    /// Pops state for redo.
    pub fn pop_redo_state(&mut self) -> Option<Vec<u8>> {
        if let Some(state) = self.redo_stack.pop() {
            if self.undo_index < self.undo_stack.len() {
                self.undo_index += 1;
            }
            return Some(state);
        }
        None
    }

    /// Clears undo/redo history.
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.undo_index = 0;
    }

    /// Returns asset paths.
    #[must_use]
    pub fn asset_paths(&self) -> &[String] {
        &self.asset_paths
    }

    /// Adds an asset path.
    pub fn add_asset_path(&mut self, path: impl Into<String>) {
        let path = path.into();
        if !self.asset_paths.contains(&path) {
            self.asset_paths.push(path);
        }
    }

    /// Removes an asset path.
    pub fn remove_asset_path(&mut self, path: &str) -> bool {
        if let Some(pos) = self.asset_paths.iter().position(|p| p == path) {
            self.asset_paths.remove(pos);
            true
        } else {
            false
        }
    }

    /// Records autosave.
    pub fn record_autosave(&mut self, path: impl Into<String>) {
        self.last_autosave = Some(AutosaveInfo {
            path:        path.into(),
            timestamp:   Timestamp::now(),
            is_recovery: false,
        });
    }

    /// Returns last autosave info.
    #[must_use]
    pub fn last_autosave(&self) -> Option<&AutosaveInfo> {
        self.last_autosave.as_ref()
    }
}

/// Recent file entry.
#[derive(Debug, Clone)]
pub struct RecentFile {
    /// File path.
    pub path:        String,
    /// Project name.
    pub name:        String,
    /// Last opened timestamp.
    pub last_opened: Timestamp,
    /// Whether file exists.
    pub exists:      bool,
    /// Thumbnail path (if available).
    pub thumbnail:   Option<String>,
}

/// Project template.
#[derive(Debug, Clone)]
pub struct ProjectTemplate {
    /// Template name.
    pub name:        String,
    /// Template description.
    pub description: String,
    /// Template category.
    pub category:    String,
    /// Project settings.
    pub settings:    ProjectSettings,
    /// Preview image path.
    pub preview:     Option<String>,
    /// Whether this is a built-in template.
    pub builtin:     bool,
}

impl ProjectTemplate {
    /// Creates a built-in 1080p template.
    #[must_use]
    pub fn hd_1080p() -> Self {
        Self {
            name:        "HD 1080p".into(),
            description: "Standard 1920x1080 HD project".into(),
            category:    "Video".into(),
            settings:    ProjectSettings::default(),
            preview:     None,
            builtin:     true,
        }
    }

    /// Creates a built-in 4K template.
    #[must_use]
    pub fn uhd_4k() -> Self {
        Self {
            name:        "4K UHD".into(),
            description: "3840x2160 Ultra HD project".into(),
            category:    "Video".into(),
            settings:    ProjectSettings::uhd_4k(),
            preview:     None,
            builtin:     true,
        }
    }

    /// Creates a film template.
    #[must_use]
    pub fn film_24p() -> Self {
        Self {
            name:        "Film 24fps".into(),
            description: "Cinema-style 24fps project".into(),
            category:    "Film".into(),
            settings:    ProjectSettings::film(),
            preview:     None,
            builtin:     true,
        }
    }

    /// Creates a social media template.
    #[must_use]
    pub fn social_square() -> Self {
        Self {
            name:        "Social Square".into(),
            description: "1080x1080 square format for social media".into(),
            category:    "Social".into(),
            settings:    ProjectSettings {
                timeline_width: 1080,
                timeline_height: 1080,
                ..Default::default()
            },
            preview:     None,
            builtin:     true,
        }
    }

    /// Creates a vertical video template.
    #[must_use]
    pub fn social_vertical() -> Self {
        Self {
            name:        "Social Vertical".into(),
            description: "1080x1920 vertical format for stories/reels".into(),
            category:    "Social".into(),
            settings:    ProjectSettings {
                timeline_width: 1080,
                timeline_height: 1920,
                ..Default::default()
            },
            preview:     None,
            builtin:     true,
        }
    }
}

/// Manager for project operations.
pub struct ProjectManager {
    /// Current project.
    current_project:     Option<Project>,
    /// Recent files.
    recent_files:        Vec<RecentFile>,
    /// Available templates.
    templates:           Vec<ProjectTemplate>,
    /// Next project ID.
    next_id:             u64,
    /// Maximum recent files.
    max_recent:          usize,
    /// Autosave enabled.
    autosave_enabled:    bool,
    /// Last autosave check time.
    last_autosave_check: Option<Timestamp>,
}

impl ProjectManager {
    /// Creates a new project manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_project:     None,
            recent_files:        Vec::new(),
            templates:           Self::builtin_templates(),
            next_id:             1,
            max_recent:          20,
            autosave_enabled:    true,
            last_autosave_check: None,
        }
    }

    /// Returns built-in templates.
    fn builtin_templates() -> Vec<ProjectTemplate> {
        vec![
            ProjectTemplate::hd_1080p(),
            ProjectTemplate::uhd_4k(),
            ProjectTemplate::film_24p(),
            ProjectTemplate::social_square(),
            ProjectTemplate::social_vertical(),
        ]
    }

    /// Generates a new project ID.
    fn next_id(&mut self) -> ProjectId {
        let id = ProjectId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// Creates a new project.
    pub fn new_project(&mut self, name: impl Into<String>) -> VideoEditorResult<&mut Project> {
        if let Some(project) = &self.current_project
            && project.has_unsaved_changes()
        {
            return Err(VideoEditorError::Io(
                "Current project has unsaved changes".into(),
            ));
        }

        let id = self.next_id();
        self.current_project = Some(Project::new(id, name));

        self.current_project
            .as_mut()
            .ok_or_else(|| VideoEditorError::Io("Failed to create project".into()))
    }

    /// Creates a project from a template.
    pub fn new_from_template(
        &mut self, template_name: &str, project_name: impl Into<String>,
    ) -> VideoEditorResult<&mut Project> {
        let template = self
            .templates
            .iter()
            .find(|t| t.name == template_name)
            .ok_or_else(|| VideoEditorError::Io(format!("Template not found: {template_name}")))?
            .clone();

        if let Some(project) = &self.current_project
            && project.has_unsaved_changes()
        {
            return Err(VideoEditorError::Io(
                "Current project has unsaved changes".into(),
            ));
        }

        let id = self.next_id();
        let mut project = Project::new(id, project_name);
        project.settings = template.settings;

        self.current_project = Some(project);

        self.current_project
            .as_mut()
            .ok_or_else(|| VideoEditorError::Io("Failed to create project".into()))
    }

    /// Returns the current project.
    #[must_use]
    pub fn current_project(&self) -> Option<&Project> {
        self.current_project.as_ref()
    }

    /// Returns the current project mutably.
    pub fn current_project_mut(&mut self) -> Option<&mut Project> {
        self.current_project.as_mut()
    }

    /// Closes the current project.
    pub fn close_project(&mut self) -> VideoEditorResult<()> {
        if let Some(project) = &self.current_project
            && project.has_unsaved_changes()
        {
            return Err(VideoEditorError::Io("Project has unsaved changes".into()));
        }
        self.current_project = None;
        Ok(())
    }

    /// Adds a file to recent files.
    pub fn add_recent(&mut self, path: impl Into<String>, name: impl Into<String>) {
        let path = path.into();
        let name = name.into();

        // Remove if already exists
        self.recent_files.retain(|r| r.path != path);

        // Add to front
        self.recent_files.insert(0, RecentFile {
            path,
            name,
            last_opened: Timestamp::now(),
            exists: true,
            thumbnail: None,
        });

        // Trim to max
        if self.recent_files.len() > self.max_recent {
            self.recent_files.truncate(self.max_recent);
        }
    }

    /// Returns recent files.
    #[must_use]
    pub fn recent_files(&self) -> &[RecentFile] {
        &self.recent_files
    }

    /// Clears recent files.
    pub fn clear_recent(&mut self) {
        self.recent_files.clear();
    }

    /// Returns available templates.
    #[must_use]
    pub fn templates(&self) -> &[ProjectTemplate] {
        &self.templates
    }

    /// Adds a custom template.
    pub fn add_template(&mut self, template: ProjectTemplate) {
        self.templates.push(template);
    }

    /// Checks if autosave is needed.
    #[must_use]
    pub fn needs_autosave(&self) -> bool {
        if !self.autosave_enabled {
            return false;
        }

        let Some(project) = &self.current_project else {
            return false;
        };

        if !project.has_unsaved_changes() {
            return false;
        }

        let interval = project.settings().autosave_interval;
        if interval == 0 {
            return false;
        }

        let Some(last_check) = self.last_autosave_check else {
            return true;
        };

        let elapsed = Timestamp::now().elapsed_since(last_check).as_secs();
        elapsed >= interval as u64
    }

    /// Updates autosave check time.
    pub fn update_autosave_check(&mut self) {
        self.last_autosave_check = Some(Timestamp::now());
    }

    /// Enables or disables autosave.
    pub fn set_autosave_enabled(&mut self, enabled: bool) {
        self.autosave_enabled = enabled;
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let mut manager = ProjectManager::new();
        let project = manager.new_project("Test Project").ok();

        assert!(project.is_some());
        assert!(manager.current_project().is_some());
    }

    #[test]
    fn test_project_from_template() {
        let mut manager = ProjectManager::new();
        let result = manager.new_from_template("HD 1080p", "My Project");

        assert!(result.is_ok());
        let project = manager.current_project().expect("unwrap conversion");
        assert_eq!(project.settings().timeline_width, 1920);
        assert_eq!(project.settings().timeline_height, 1080);
    }

    #[test]
    fn test_recent_files() {
        let mut manager = ProjectManager::new();
        manager.add_recent("/path/to/project1.proj", "Project 1");
        manager.add_recent("/path/to/project2.proj", "Project 2");

        assert_eq!(manager.recent_files().len(), 2);
        assert_eq!(manager.recent_files()[0].name, "Project 2"); // Most recent first
    }

    #[test]
    fn test_project_undo() {
        let id = ProjectId::new(1);
        let mut project = Project::new(id, "Test");

        project.push_undo_state(vec![1, 2, 3]);
        project.push_undo_state(vec![4, 5, 6]);

        assert!(project.can_undo());
        assert!(!project.can_redo());

        let state = project.pop_undo_state();
        assert!(state.is_some());
        assert!(project.can_redo());
    }

    #[test]
    fn test_templates() {
        let manager = ProjectManager::new();
        assert!(manager.templates().len() >= 4); // At least the built-in templates
    }
}
