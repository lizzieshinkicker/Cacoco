use crate::document::actions::SBarAction;
use crate::models::sbardef::{SBarDefFile, StatusBarLayout};
use std::collections::HashSet;

/// Processes actions that target the top-level Status Bar layout list.
pub fn execute_layout_action(
    file: &mut SBarDefFile,
    action: SBarAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        SBarAction::AddStatusBar => {
            file.data.status_bars.push(StatusBarLayout::default());
            let new_idx = file.data.status_bars.len() - 1;
            selection.clear();
            selection.insert(vec![new_idx]);
        }
        SBarAction::DuplicateStatusBar(idx) => {
            if let Some(bar) = file.data.status_bars.get(idx) {
                let mut new_bar = bar.clone();
                new_bar.reassign_all_uids();
                file.data.status_bars.insert(idx + 1, new_bar);
                selection.clear();
                selection.insert(vec![idx + 1]);
            }
        }
        SBarAction::MoveStatusBar { source, target } => {
            if source < file.data.status_bars.len() {
                let element = file.data.status_bars.remove(source);
                let mut final_target = target;
                if source < target {
                    final_target = final_target.saturating_sub(1);
                }
                let safe_target = final_target.min(file.data.status_bars.len());
                file.data.status_bars.insert(safe_target, element);
                selection.clear();
                selection.insert(vec![safe_target]);
            }
        }
        SBarAction::DeleteStatusBar(idx) => {
            if file.data.status_bars.len() > 1 && idx < file.data.status_bars.len() {
                file.data.status_bars.remove(idx);
                selection.clear();
            }
        }
        SBarAction::PasteStatusBars(bars) => {
            selection.clear();
            for mut bar in bars {
                bar.reassign_all_uids();
                file.data.status_bars.push(bar);
                selection.insert(vec![file.data.status_bars.len() - 1]);
            }
        }
    }
}
