use crate::models::{ProjectData, sbardef::ElementWrapper, sbardef::StatusBarLayout};
use std::collections::{HashSet, VecDeque};

const MAX_UNDO: usize = 50;

#[derive(Clone)]
pub struct Snapshot {
    pub file: ProjectData,
    pub selection: HashSet<Vec<usize>>,
}

pub struct HistoryManager {
    pub undo_stack: VecDeque<Snapshot>,
    pub redo_stack: VecDeque<Snapshot>,
    pub clipboard: Vec<ElementWrapper>,
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
    pub fn take_snapshot(
        &mut self,
        current_file: &ProjectData,
        current_selection: &HashSet<Vec<usize>>,
    ) {
        self.undo_stack.push_back(Snapshot {
            file: current_file.clone(),
            selection: current_selection.clone(),
        });

        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
    }

    pub fn undo(
        &mut self,
        current_file: &mut ProjectData,
        current_selection: &mut HashSet<Vec<usize>>,
    ) {
        if let Some(prev) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(Snapshot {
                file: current_file.clone(),
                selection: current_selection.clone(),
            });

            *current_file = prev.file;
            *current_selection = prev.selection;
        }
    }

    pub fn redo(
        &mut self,
        current_file: &mut ProjectData,
        current_selection: &mut HashSet<Vec<usize>>,
    ) {
        if let Some(next) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(Snapshot {
                file: current_file.clone(),
                selection: current_selection.clone(),
            });

            *current_file = next.file;
            *current_selection = next.selection;
        }
    }

    pub fn prepare_clipboard_for_paste(&self) -> Vec<ElementWrapper> {
        let mut pasted = self.clipboard.clone();
        for item in pasted.iter_mut() {
            item.reassign_uids();
        }
        pasted
    }

    pub fn prepare_bar_clipboard_for_paste(&self) -> Vec<StatusBarLayout> {
        let mut pasted = self.bar_clipboard.clone();
        for item in pasted.iter_mut() {
            item.reassign_all_uids();
        }
        pasted
    }
}
