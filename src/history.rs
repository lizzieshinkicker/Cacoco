use std::collections::{VecDeque, HashSet};
use crate::model::{SBarDefFile, ElementWrapper, new_uid};

const MAX_UNDO: usize = 50;

#[derive(Clone)]
pub struct Snapshot {
    pub file: SBarDefFile,
    pub selection: HashSet<Vec<usize>>,
}

pub struct HistoryManager {
    pub undo_stack: VecDeque<Snapshot>,
    pub redo_stack: VecDeque<Snapshot>,
    pub clipboard: Vec<ElementWrapper>,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(MAX_UNDO),
            redo_stack: VecDeque::with_capacity(MAX_UNDO),
            clipboard: Vec::new(),
        }
    }
}

impl HistoryManager {
    pub fn take_snapshot(&mut self, current_file: &SBarDefFile, current_selection: &HashSet<Vec<usize>>) {
        self.undo_stack.push_back(Snapshot {
            file: current_file.clone(),
            selection: current_selection.clone(),
        });

        if self.undo_stack.len() > MAX_UNDO {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_file: &mut SBarDefFile, current_selection: &mut HashSet<Vec<usize>>) {
        if let Some(prev) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(Snapshot {
                file: current_file.clone(),
                selection: current_selection.clone(),
            });

            *current_file = prev.file;
            *current_selection = prev.selection;
        }
    }

    pub fn redo(&mut self, current_file: &mut SBarDefFile, current_selection: &mut HashSet<Vec<usize>>) {
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
            Self::reassign_uids(item);
        }
        pasted
    }

    fn reassign_uids(element: &mut ElementWrapper) {
        element.uid = new_uid();
        for child in element.get_common_mut().children.iter_mut() {
            Self::reassign_uids(child);
        }
    }
}