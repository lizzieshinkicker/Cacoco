use crate::models::{ProjectData, sbardef::ElementWrapper, sbardef::StatusBarLayout};
use std::collections::{HashSet, VecDeque};

/// The maximum number of undo steps to keep in memory.
const MAX_UNDO: usize = 50;

/// A point-in-time capture of the entire project's state.
///
/// Captures all lumps (SBARDEF, SKYDEFS, etc.) and the user's current selection.
#[derive(Clone)]
pub struct Snapshot {
    /// All lumps present in the project at the time of the snapshot.
    pub lumps: Vec<ProjectData>,
    /// The set of paths selected by the user.
    pub selection: HashSet<Vec<usize>>,
}

/// Manages the undo/redo stacks and the application-wide clipboard.
pub struct HistoryManager {
    /// Stack of previous states for the Undo operation.
    pub undo_stack: VecDeque<Snapshot>,
    /// Stack of undone states for the Redo operation.
    pub redo_stack: VecDeque<Snapshot>,
    /// Elements copied to the clipboard (e.g., Graphics, Canvases).
    pub clipboard: Vec<ElementWrapper>,
    /// Status Bar layouts copied to the clipboard.
    pub bar_clipboard: Vec<StatusBarLayout>,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(MAX_UNDO),
            redo_stack: VecDeque::with_capacity(MAX_UNDO),
            clipboard: Vec::new(),
            bar_clipboard: Vec::new(),
        }
    }
}

impl HistoryManager {
    /// Captures the current state of all lumps and adds it to the undo stack.
    pub fn take_snapshot(
        &mut self,
        current_lumps: &Vec<ProjectData>,
        current_selection: &HashSet<Vec<usize>>,
    ) {
        self.undo_stack.push_back(Snapshot {
            lumps: current_lumps.clone(),
            selection: current_selection.clone(),
        });

        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.pop_front();
        }

        self.redo_stack.clear();
    }

    /// Reverts the project to the most recent state in the undo stack.
    pub fn undo(
        &mut self,
        current_lumps: &mut Vec<ProjectData>,
        current_selection: &mut HashSet<Vec<usize>>,
    ) {
        if let Some(prev) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(Snapshot {
                lumps: current_lumps.clone(),
                selection: current_selection.clone(),
            });

            *current_lumps = prev.lumps;
            *current_selection = prev.selection;
        }
    }

    /// Restores a state that was previously undone.
    pub fn redo(
        &mut self,
        current_lumps: &mut Vec<ProjectData>,
        current_selection: &mut HashSet<Vec<usize>>,
    ) {
        if let Some(next) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(Snapshot {
                lumps: current_lumps.clone(),
                selection: current_selection.clone(),
            });

            *current_lumps = next.lumps;
            *current_selection = next.selection;
        }
    }

    /// Deep-clones the clipboard elements and reassigns UIDs for a clean paste.
    pub fn prepare_clipboard_for_paste(&self) -> Vec<ElementWrapper> {
        let mut pasted = self.clipboard.clone();
        for item in pasted.iter_mut() {
            item.reassign_uids();
        }
        pasted
    }

    /// Deep-clones the clipboard layouts and reassigns all internal UIDs.
    pub fn prepare_bar_clipboard_for_paste(&self) -> Vec<StatusBarLayout> {
        let mut pasted = self.bar_clipboard.clone();
        for item in pasted.iter_mut() {
            item.reassign_all_uids();
        }
        pasted
    }
}
