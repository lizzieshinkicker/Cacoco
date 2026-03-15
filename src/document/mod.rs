pub mod actions;
mod layout;
mod sky;
mod tree;
mod umapinfo;

pub(crate) use crate::document::actions::{DocumentAction, TreeAction};
pub use tree::determine_insertion_point;

use crate::app::ProjectMode;
use crate::history::HistoryManager;
use crate::models::ProjectData;
use std::collections::HashSet;

/// Manages a collection of ID24 lumps, their selection, and its modification history.
pub struct ProjectDocument {
    /// All lumps contained in this project that Cacoco understands (SBARDEF, SKYDEFS, etc.)
    pub lumps: Vec<ProjectData>,
    /// All lumps contained in this project that Cacoco passes through (MAPs and whatnot)
    pub passthrough_lumps: Vec<crate::wad::RawLump>,
    /// The filesystem path where this document is saved.
    pub path: Option<String>,
    /// The set of tree-paths currently selected by the user.
    pub selection: HashSet<Vec<usize>>,
    /// The anchor path used for shift-selection logic.
    pub selection_pivot: Option<Vec<usize>>,
    /// The undo and redo history for this specific document.
    pub history: HistoryManager,
    /// True if the document has modifications that haven't been saved.
    pub dirty: bool,
}

impl ProjectDocument {
    /// Creates a new document from an initial lump and optional path.
    pub fn new(
        initial_lump: ProjectData,
        passthrough: Vec<crate::wad::RawLump>,
        path: Option<String>,
    ) -> Self {
        Self {
            lumps: vec![initial_lump],
            passthrough_lumps: passthrough,
            path,
            selection: HashSet::new(),
            selection_pivot: None,
            history: HistoryManager::default(),
            dirty: false,
        }
    }

    /// Returns a mutable reference to the lump matching the given mode.
    pub fn get_lump_mut(&mut self, mode: ProjectMode) -> Option<&mut ProjectData> {
        self.lumps
            .iter_mut()
            .find(|l| ProjectMode::from_data(l) == mode)
    }

    /// Returns an immutable reference to the lump matching the given mode.
    pub fn get_lump(&self, mode: ProjectMode) -> Option<&ProjectData> {
        self.lumps
            .iter()
            .find(|l| ProjectMode::from_data(l) == mode)
    }

    /// Primary entry point for mutating project data via user actions.
    pub fn execute_actions(&mut self, actions: Vec<DocumentAction>, active_mode: ProjectMode) {
        for action in actions {
            match action {
                DocumentAction::UndoSnapshot => {
                    self.history.take_snapshot(&self.lumps, &self.selection);
                }
                _ => {
                    self.dirty = true;
                    let selection_ref = &mut self.selection;
                    let active_lump = self
                        .lumps
                        .iter_mut()
                        .find(|l| ProjectMode::from_data(l) == active_mode);

                    match action {
                        DocumentAction::SBar(sbar_act) => {
                            if let Some(ProjectData::StatusBar(sbar)) = active_lump {
                                layout::execute_layout_action(sbar, sbar_act, selection_ref);
                            }
                        }
                        DocumentAction::Tree(tree_act) => {
                            if let Some(ProjectData::StatusBar(sbar)) = active_lump {
                                tree::execute_tree_action(sbar, tree_act, selection_ref);
                            } else if let Some(ProjectData::UmapInfo(_)) = active_lump {
                                if let TreeAction::Select(paths) = tree_act {
                                    selection_ref.clear();
                                    for path in paths {
                                        selection_ref.insert(path);
                                    }
                                }
                            }
                        }
                        DocumentAction::Sky(sky_act) => {
                            if let Some(ProjectData::Sky(sky_file)) = active_lump {
                                sky::execute_sky_action(sky_file, sky_act, selection_ref);
                            }
                        }
                        DocumentAction::Umap(umap_act) => {
                            if let Some(ProjectData::UmapInfo(info)) = active_lump {
                                umapinfo::execute_umapinfo_action(info, umap_act, selection_ref);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if let Some(ProjectData::StatusBar(sbar)) = self.get_lump_mut(active_mode) {
            sbar.normalize_for_target();
        }
    }

    /// Reverts the document to the previous state in history.
    pub fn undo(&mut self) {
        self.history.undo(&mut self.lumps, &mut self.selection);
        self.dirty = true;
    }

    /// Re-applies a state that was recently undone.
    pub fn redo(&mut self) {
        self.history.redo(&mut self.lumps, &mut self.selection);
        self.dirty = true;
    }
}
