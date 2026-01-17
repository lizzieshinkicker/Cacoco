mod layout;
mod tree;

use crate::app::ProjectMode;
use crate::history::HistoryManager;
use crate::models::ProjectData;
use crate::models::sbardef::{ElementWrapper, StatusBarLayout};
use std::collections::HashSet;

/// Actions that can be performed on the document's layer hierarchy or layouts.
#[derive(Debug, Clone)]
pub enum LayerAction {
    /// Forces an undo snapshot to be taken before the next mutation.
    UndoSnapshot,
    /// Deletes the specified paths from the tree.
    DeleteSelection(Vec<Vec<usize>>),
    /// Duplicates the specified paths at their current locations.
    DuplicateSelection(Vec<Vec<usize>>),
    /// Moves an element one index up within its parent list.
    MoveUp(Vec<usize>),
    /// Moves an element one index down within its parent list.
    MoveDown(Vec<usize>),
    /// Performs a move operation, potentially changing the parent of the elements.
    MoveSelection {
        sources: Vec<Vec<usize>>,
        target_parent: Vec<usize>,
        insert_idx: usize,
    },
    /// Adds a new element at the specified location.
    Add {
        parent_path: Vec<usize>,
        insert_idx: usize,
        element: ElementWrapper,
    },
    /// Pastes a list of elements from the clipboard.
    Paste {
        parent_path: Vec<usize>,
        insert_idx: usize,
        elements: Vec<ElementWrapper>,
    },
    /// Offsets the X/Y coordinates of the specified elements.
    TranslateSelection {
        paths: Vec<Vec<usize>>,
        dx: i32,
        dy: i32,
    },
    /// Adds a new status bar layout to the project.
    AddStatusBar,
    /// Duplicates an existing status bar layout.
    DuplicateStatusBar(usize),
    /// Reorders status bar layouts.
    MoveStatusBar { source: usize, target: usize },
    /// Deletes a status bar layout.
    DeleteStatusBar(usize),
    /// Pastes layouts from the clipboard.
    PasteStatusBars(Vec<StatusBarLayout>),
    /// Wraps the current selection in a new Canvas group.
    GroupSelection(Vec<Vec<usize>>),
    /// Replaces the current selection with a new set of paths.
    Select(Vec<Vec<usize>>),
    /// Toggles the selection state of specific paths (Multi-select support).
    ToggleSelection(Vec<Vec<usize>>),
}

/// Manages a collection of ID24 lumps, their selection, and its modification history.
pub struct ProjectDocument {
    /// All lumps contained in this project (SBARDEF, SKYDEFS, etc.)
    pub lumps: Vec<ProjectData>,
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
    pub fn new(initial_lump: ProjectData, path: Option<String>) -> Self {
        Self {
            lumps: vec![initial_lump],
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
    pub fn execute_actions(&mut self, actions: Vec<LayerAction>, active_mode: ProjectMode) {
        for action in actions {
            match action {
                LayerAction::UndoSnapshot => {
                    self.history.take_snapshot(&self.lumps, &self.selection);
                }
                _ => {
                    self.dirty = true;
                    let selection_ref = &mut self.selection;
                    let active_lump = self
                        .lumps
                        .iter_mut()
                        .find(|l| ProjectMode::from_data(l) == active_mode);

                    if let Some(ProjectData::StatusBar(sbar)) = active_lump {
                        match action {
                            LayerAction::AddStatusBar
                            | LayerAction::DuplicateStatusBar(_)
                            | LayerAction::MoveStatusBar { .. }
                            | LayerAction::DeleteStatusBar(_)
                            | LayerAction::PasteStatusBars(_) => {
                                layout::execute_layout_action(sbar, action, selection_ref);
                            }
                            _ => {
                                tree::execute_tree_action(sbar, action, selection_ref);
                            }
                        }
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

/// Re-export helper for external UI components.
pub use tree::determine_insertion_point;
