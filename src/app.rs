use crate::assets::AssetStore;
use crate::cheats::CheatEngine;
use crate::config::AppConfig;
use crate::history::HistoryManager;
use crate::hotkeys::HotkeyRegistry;
use crate::io;
use crate::library::Template;
use crate::model::SBarDefFile;
use crate::state::PreviewState;
use crate::ui;
use crate::ui::font_wizard::FontWizardState;
use eframe::egui;
use std::collections::HashSet;

const MAX_RECENT_FILES: usize = 5;

#[derive(Clone)]
pub enum ConfirmationRequest {
    DeleteStatusBar(usize),
    DeleteAssets(Vec<String>),
}

pub struct CacocoApp {
    pub current_file: Option<SBarDefFile>,
    pub opened_file_path: Option<String>,
    pub selection: HashSet<Vec<usize>>,
    pub last_selection: HashSet<Vec<usize>>,
    pub selection_pivot: Option<Vec<usize>>,
    pub assets: AssetStore,
    pub config: AppConfig,
    pub preview_state: PreviewState,
    pub cheat_engine: CheatEngine,
    pub current_statusbar_idx: usize,
    pub settings_open: bool,
    pub font_wizard: Option<FontWizardState>,
    pub confirmation_modal: Option<ConfirmationRequest>,
    pub history: HistoryManager,
    pub hotkeys: HotkeyRegistry,
    pub iwad_verified: bool,
}

impl Default for CacocoApp {
    fn default() -> Self {
        Self {
            current_file: None,
            opened_file_path: None,
            selection: HashSet::new(),
            last_selection: HashSet::new(),
            selection_pivot: None,
            assets: AssetStore::default(),
            config: AppConfig::load(),
            preview_state: PreviewState::default(),
            cheat_engine: CheatEngine::default(),
            current_statusbar_idx: 0,
            settings_open: false,
            font_wizard: None,
            confirmation_modal: None,
            history: HistoryManager::default(),
            hotkeys: HotkeyRegistry::default(),
            iwad_verified: false,
        }
    }
}

impl CacocoApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let mut app = Self::default();
        app.load_system_assets(&cc.egui_ctx);

        if let Some(path) = &app.config.base_wad_path {
            app.iwad_verified = io::load_wad_from_path(&cc.egui_ctx, path, &mut app.assets);
        }

        app
    }

    pub fn load_system_assets(&mut self, ctx: &egui::Context) {
        self.assets.load_smooth_image(
            ctx,
            "_BG_MASTER",
            include_bytes!("../assets/background.png"),
        );
        self.assets.load_reference_image(
            ctx,
            "HICACOCO",
            include_bytes!("../assets/HICACOCO.png"),
        );
        self.assets.load_system_assets(ctx);

        for asset in crate::library::ASSETS {
            let key = AssetStore::stem(asset.name);
            self.assets.load_reference_image(ctx, &key, asset.bytes);
        }
    }

    pub fn add_to_recent(&mut self, path: &str) {
        self.config.recent_files.retain(|p| p != path);
        self.config.recent_files.push_front(path.to_string());
        self.config.recent_files.truncate(MAX_RECENT_FILES);
        self.config.save();
    }

    pub fn load_project(&mut self, ctx: &egui::Context, loaded: io::LoadedProject, path_str: &str) {
        self.current_file = Some(loaded.file);
        self.opened_file_path = Some(path_str.to_string());
        self.assets = loaded.assets;

        self.preview_state = PreviewState::default();

        self.load_system_assets(ctx);

        if let Some(path) = &self.config.base_wad_path {
            io::load_wad_from_path(ctx, path, &mut self.assets);
        }

        self.selection.clear();
        self.last_selection.clear();
        self.selection_pivot = None;
        self.current_statusbar_idx = 0;
        self.add_to_recent(path_str);

        self.history.undo_stack.clear();
        self.history.redo_stack.clear();

        self.preview_state
            .push_message(format!("Project Loaded: {}", path_str));
    }

    pub fn new_project(&mut self, ctx: &egui::Context) {
        self.current_file = Some(crate::model::SBarDefFile {
            type_: "sbardef".to_string(),
            version: "1.0".to_string(),
            data: crate::model::StatusBarDefinition {
                status_bars: vec![crate::model::StatusBarLayout::default()],
                ..Default::default()
            },
        });

        self.opened_file_path = Some("NewStatusBar.pk3".to_string());
        self.assets = AssetStore::default();

        self.preview_state = PreviewState::default();

        self.load_system_assets(ctx);

        if let Some(path) = &self.config.base_wad_path {
            io::load_wad_from_path(ctx, path, &mut self.assets);
        }

        self.selection.clear();
        self.last_selection.clear();
        self.selection_pivot = None;
        self.current_statusbar_idx = 0;
        self.history.undo_stack.clear();
        self.history.redo_stack.clear();
        self.preview_state.push_message("Created new empty project.");
    }

    pub fn apply_template(&mut self, ctx: &egui::Context, template: &Template) {
        match serde_json::from_str::<SBarDefFile>(template.json_content) {
            Ok(parsed_file) => {
                self.current_file = Some(parsed_file);
                self.opened_file_path = Some(format!("{}.pk3", template.name));

                self.assets = AssetStore::default();

                self.preview_state = PreviewState::default();

                self.load_system_assets(ctx);
                if let Some(path) = &self.config.base_wad_path {
                    io::load_wad_from_path(ctx, path, &mut self.assets);
                }

                self.selection.clear();
                self.last_selection.clear();
                self.selection_pivot = None;
                self.current_statusbar_idx = 0;
                self.history.undo_stack.clear();
                self.history.redo_stack.clear();

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
            Err(e) => {
                eprintln!("Failed to parse template JSON: {}", e);
            }
        }
    }
}

impl eframe::App for CacocoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::dark());

        if self.selection != self.last_selection {
            self.preview_state.strobe_timer = 0.5;
            self.last_selection = self.selection.clone();
        }

        ui::draw_root_ui(ctx, self);
    }
}