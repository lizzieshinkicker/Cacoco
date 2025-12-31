use crate::assets::AssetStore;
use crate::cheats::CheatEngine;
use crate::config::AppConfig;
use crate::document::{LayerAction, SBarDocument};
use crate::io;
use crate::library::Template;
use crate::model::SBarDefFile;
use crate::state::PreviewState;
use crate::ui;
use crate::ui::font_wizard::FontWizardState;
use crate::ui::viewport_controller::ViewportController;
use eframe::egui;
use std::collections::HashSet;

const MAX_RECENT_FILES: usize = 5;

/// Actions that require user confirmation (usually due to unsaved changes).
#[derive(Clone)]
pub enum PendingAction {
    New,
    Load(String),
    Template(&'static Template),
    Quit,
}

/// Requests for user confirmation before performing destructive operations.
#[derive(Clone)]
pub enum ConfirmationRequest {
    DeleteStatusBar(usize),
    DeleteLayers(Vec<Vec<usize>>),
    DeleteAssets(Vec<String>),
    DiscardChanges(PendingAction),
}

/// The main application container for the Cacoco SBARDEF editor.
pub struct CacocoApp {
    /// The active project document, if any.
    pub doc: Option<SBarDocument>,
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
        }
    }
}

impl CacocoApp {
    /// Creates a new instance of the application and loads initial assets.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let mut app = Self::default();
        app.load_system_assets(&cc.egui_ctx);

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
        self.doc = Some(SBarDocument::new(loaded.file, Some(path_str.to_string())));
        self.assets = loaded.assets;
        self.preview_state = PreviewState::default();

        self.load_system_assets(ctx);
        if let Some(path) = &self.config.base_wad_path {
            io::load_wad_from_path(ctx, path, &mut self.assets);
        }

        self.last_selection.clear();
        self.current_statusbar_idx = 0;
        self.add_to_recent(path_str);

        self.preview_state
            .push_message(format!("Project Loaded: {}", path_str));
    }

    /// Initializes a new empty SBARDEF project.
    pub fn new_project(&mut self, ctx: &egui::Context) {
        let file = SBarDefFile {
            type_: "statusbar".to_string(),
            version: "1.0.0".to_string(),
            data: crate::model::StatusBarDefinition {
                status_bars: vec![crate::model::StatusBarLayout::default()],
                ..Default::default()
            },
        };

        self.doc = Some(SBarDocument::new(file, None));
        self.assets = AssetStore::default();
        self.preview_state = PreviewState::default();

        self.load_system_assets(ctx);
        if let Some(path) = &self.config.base_wad_path {
            io::load_wad_from_path(ctx, path, &mut self.assets);
        }

        self.last_selection.clear();
        self.current_statusbar_idx = 0;
        self.preview_state
            .push_message("Created new empty project.");
    }

    /// Applies a library template as the current project.
    pub fn apply_template(&mut self, ctx: &egui::Context, template: &Template) {
        match serde_json::from_str::<SBarDefFile>(template.json_content) {
            Ok(parsed_file) => {
                self.doc = Some(SBarDocument::new(parsed_file, None));
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
                self.preview_state
                    .push_message(format!("Template: {}", template.name));
            }
            Err(e) => eprintln!("Failed to parse template JSON: {}", e),
        }
    }

    /// Delegation helper to execute actions on the current document.
    pub fn execute_actions(&mut self, actions: Vec<LayerAction>) {
        if let Some(doc) = &mut self.doc {
            let old_selection = doc.selection.clone();
            doc.execute_actions(actions);

            if doc.selection != old_selection {
                self.preview_state.editor.strobe_timer = 0.5;
            }

            if doc.selection.len() == 1 {
                let path = doc.selection.iter().next().unwrap();
                if path.len() == 1 {
                    self.current_statusbar_idx = path[0];
                }
            }
        }

        if let Some(doc) = &self.doc {
            if self.current_statusbar_idx >= doc.file.data.status_bars.len() {
                self.current_statusbar_idx = doc.file.data.status_bars.len().saturating_sub(1);
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
    }
}
