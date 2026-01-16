use crate::assets::AssetStore;
use crate::cheats::CheatEngine;
use crate::config::AppConfig;
use crate::document::{LayerAction, ProjectDocument};
use crate::io;
use crate::state::PreviewState;
use crate::ui;
use crate::ui::font_wizard::FontWizardState;
use crate::ui::messages::{self, EditorEvent};
use crate::ui::viewport_controller::ViewportController;
use eframe::egui;
use std::collections::HashSet;

const MAX_RECENT_FILES: usize = 5;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum ProjectMode {
    #[default]
    SBarDef,
    SkyDefs,
    Interlevel,
    Finale,
}

impl ProjectMode {
    pub fn from_data(data: &crate::models::ProjectData) -> Self {
        match data {
            crate::models::ProjectData::StatusBar(_) => ProjectMode::SBarDef,
            crate::models::ProjectData::Finale(_) => ProjectMode::Finale,
            crate::models::ProjectData::Sky(_) => ProjectMode::SkyDefs,
            crate::models::ProjectData::Interlevel(_) => ProjectMode::Interlevel,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CreationModal {
    #[default]
    None,
    LumpSelector,
    SBarDef,
    SkyDefs,
    Interlevel,
    Finale,
}

/// Actions that require user confirmation (usually due to unsaved changes).
#[derive(Clone)]
pub enum PendingAction {
    New,
    Load(String),
    Quit,
}

/// Requests for user confirmation before performing destructive operations.
#[derive(Clone)]
#[allow(dead_code)]
pub enum ConfirmationRequest {
    DeleteStatusBar(usize),
    DeleteLayers(Vec<Vec<usize>>),
    DeleteAssets(Vec<String>),
    DiscardChanges(PendingAction),
    DowngradeTarget(crate::models::sbardef::ExportTarget),
}

/// The main application container for the Cacoco SBARDEF editor.
pub struct CacocoApp {
    /// The active project document, if any.
    pub doc: Option<ProjectDocument>,
    /// Registry of all loaded textures and raw asset data.
    pub assets: AssetStore,
    /// Persistent application configuration (IWAD paths, recent files).
    pub config: AppConfig,
    /// The simulated game state used to render the viewport preview.
    pub preview_state: PreviewState,
    /// Controller for handling viewport interactions and dragging.
    pub viewport_ctrl: ViewportController,
    /// Logic for handling cheat codes and weapon hotkeys.
    pub cheat_engine: CheatEngine,
    /// Index of the status bar layout currently being edited.
    pub current_statusbar_idx: usize,
    /// Cached selection from the previous frame used for strobe triggers.
    pub last_selection: HashSet<Vec<usize>>,
    /// Controls the visibility of the settings modal.
    pub settings_open: bool,
    /// State for the font auto-detection wizard.
    pub font_wizard: Option<FontWizardState>,
    /// State for any active confirmation dialog.
    pub confirmation_modal: Option<ConfirmationRequest>,
    /// Registry for global keyboard shortcuts.
    pub hotkeys: crate::hotkeys::HotkeyRegistry,
    /// True if a valid Doom II IWAD has been verified.
    pub iwad_verified: bool,
    /// Lump that is currently being edited.
    pub active_mode: ProjectMode,
    /// Modal to select which Lump to create.
    pub creation_modal: CreationModal,
}

impl Default for CacocoApp {
    fn default() -> Self {
        Self {
            doc: None,
            assets: AssetStore::default(),
            config: AppConfig::load(),
            preview_state: PreviewState::default(),
            viewport_ctrl: ViewportController::default(),
            cheat_engine: CheatEngine::default(),
            current_statusbar_idx: 0,
            last_selection: HashSet::new(),
            settings_open: false,
            font_wizard: None,
            confirmation_modal: None,
            hotkeys: crate::hotkeys::HotkeyRegistry::default(),
            iwad_verified: false,
            active_mode: ProjectMode::SBarDef,
            creation_modal: CreationModal::default(),
        }
    }
}

impl CacocoApp {
    /// Creates a new instance of the application and loads initial assets.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let mut app = Self::default();
        app.load_system_assets(&cc.egui_ctx);

        if app.config.base_wad_path.is_none() {
            if let Some(auto_path) = crate::discovery::find_iwad() {
                let path_str = auto_path.to_string_lossy().to_string();
                app.config.base_wad_path = Some(path_str.clone());
                app.config.save();
                println!("Auto-discovered IWAD at: {}", path_str);
            }
        }

        if let Some(path) = &app.config.base_wad_path {
            app.iwad_verified = io::load_wad_from_path(&cc.egui_ctx, path, &mut app.assets);
        }

        app
    }

    /// Loads built-in branding, badges, and template assets into the store.
    pub fn load_system_assets(&mut self, ctx: &egui::Context) {
        self.assets.load_smooth_image(
            ctx,
            "_BG_MASTER",
            include_bytes!("../assets/background.png"),
        );
        self.assets
            .load_reference_image(ctx, "HICACOCO", include_bytes!("../assets/HICACOCO.png"));
        self.assets.load_system_assets(ctx);

        for asset in crate::library::ASSETS {
            let key = AssetStore::stem(asset.name);
            self.assets.load_reference_image(ctx, &key, asset.bytes);
        }
    }

    /// Updates the recent files list and saves the configuration.
    pub fn add_to_recent(&mut self, path: &str) {
        self.config.recent_files.retain(|p| p != path);
        self.config.recent_files.push_front(path.to_string());
        self.config.recent_files.truncate(MAX_RECENT_FILES);
        self.config.save();
    }

    /// Loads a project from a file and resets the application state.
    pub fn load_project(&mut self, ctx: &egui::Context, loaded: io::LoadedProject, path_str: &str) {
        self.active_mode = ProjectMode::from_data(&loaded.file);
        self.doc = Some(ProjectDocument::new(
            loaded.file,
            Some(path_str.to_string()),
        ));
        self.assets = loaded.assets;
        self.preview_state = PreviewState::default();

        self.load_system_assets(ctx);
        if let Some(path) = &self.config.base_wad_path {
            io::load_wad_from_path(ctx, path, &mut self.assets);
        }

        self.last_selection.clear();
        self.current_statusbar_idx = 0;
        self.add_to_recent(path_str);

        messages::log_event(
            &mut self.preview_state,
            EditorEvent::ProjectLoaded(path_str.to_string()),
        );
    }

    /// Initializes a new empty project.
    pub fn new_project(&mut self, ctx: &egui::Context, data: crate::models::ProjectData) {
        self.active_mode = ProjectMode::from_data(&data);
        self.doc = Some(ProjectDocument::new(data, None));
        self.assets = AssetStore::default();
        self.preview_state = PreviewState::default();

        self.load_system_assets(ctx);
        if let Some(path) = &self.config.base_wad_path {
            io::load_wad_from_path(ctx, path, &mut self.assets);
        }

        self.last_selection.clear();
        self.current_statusbar_idx = 0;

        messages::log_event(&mut self.preview_state, EditorEvent::ProjectNew);
    }

    /// Appends a new lump to the current project or switches to it if it exists.
    pub fn add_lump_to_project(&mut self, data: crate::models::ProjectData) {
        if let Some(doc) = &mut self.doc {
            let mode = ProjectMode::from_data(&data);
            if doc.get_lump(mode).is_none() {
                doc.lumps.push(data);
                doc.dirty = true;
            }
            self.active_mode = mode;
            self.last_selection.clear();
        }
    }

    /// Applies a library template as the current project.
    pub fn apply_template(&mut self, ctx: &egui::Context, template: &crate::library::Template) {
        match serde_json::from_str::<crate::models::sbardef::SBarDefFile>(template.json_content) {
            Ok(mut parsed_file) => {
                parsed_file.normalize_paths();
                parsed_file.target = parsed_file.determine_target();
                parsed_file.normalize_for_target();

                let data = crate::models::ProjectData::StatusBar(parsed_file);
                self.active_mode = ProjectMode::from_data(&data);
                self.doc = Some(ProjectDocument::new(data, None));

                self.assets = AssetStore::default();
                self.preview_state = PreviewState::default();

                self.load_system_assets(ctx);
                if let Some(path) = &self.config.base_wad_path {
                    io::load_wad_from_path(ctx, path, &mut self.assets);
                }

                self.last_selection.clear();
                self.current_statusbar_idx = 0;

                for prefix in template.required_prefixes {
                    for lib_asset in crate::library::ASSETS {
                        if lib_asset.name.to_lowercase().starts_with(prefix) {
                            let key = AssetStore::stem(lib_asset.name);
                            self.assets.load_image(ctx, &key, lib_asset.bytes);
                        }
                    }
                }

                messages::log_event(
                    &mut self.preview_state,
                    EditorEvent::TemplateApplied(template.name.to_string()),
                );
            }
            Err(e) => eprintln!("Failed to parse template JSON: {}", e),
        }
    }

    /// Delegation helper to execute actions on the current document.
    pub fn execute_actions(&mut self, actions: Vec<LayerAction>) {
        if let Some(doc) = &mut self.doc {
            let old_selection = doc.selection.clone();
            doc.execute_actions(actions, self.active_mode);

            if doc.selection != old_selection {
                self.preview_state.editor.strobe_timer = 0.5;
            }

            if doc.selection.len() == 1 {
                let path = doc.selection.iter().next().unwrap();
                if path.len() == 1 && self.active_mode == ProjectMode::SBarDef {
                    self.current_statusbar_idx = path[0];
                }
            }
        }
    }

    /// Opens the system dialog to pick a project and loads it if successful.
    pub fn open_project_ui(&mut self, ctx: &egui::Context) {
        if let Some(path) = io::open_project_dialog() {
            if let Some(loaded) = io::load_project_from_path(ctx, &path) {
                self.load_project(ctx, loaded, &path);
            }
        }
    }
}

impl eframe::App for CacocoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        ui::draw_root_ui(ctx, self);

        if let Some(doc) = &self.doc {
            if doc.selection != self.last_selection {
                self.preview_state.editor.strobe_timer = 0.5;
                self.last_selection = doc.selection.clone();
            }
        } else if !self.last_selection.is_empty() {
            self.last_selection.clear();
        }
    }
}
