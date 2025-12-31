use crate::document::LayerAction;
use crate::model::{SBarDefFile, StatusBarLayout};
use std::collections::HashSet;

/// Processes actions that target the top-level Status Bar layout list.
pub fn execute_layout_action(
    file: &mut SBarDefFile,
    action: LayerAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        LayerAction::AddStatusBar => {
            file.data.status_bars.push(StatusBarLayout::default());
            let new_idx = file.data.status_bars.len() - 1;
            selection.clear();
            selection.insert(vec![new_idx]);
        }
        LayerAction::DuplicateStatusBar(idx) => {
            if let Some(bar) = file.data.status_bars.get(idx) {
                let mut new_bar = bar.clone();
                new_bar.reassign_all_uids();
                file.data.status_bars.insert(idx + 1, new_bar);
                selection.clear();
                selection.insert(vec![idx + 1]);
            }
        }
        LayerAction::MoveStatusBar { source, target } => {
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
        LayerAction::DeleteStatusBar(idx) => {
            if file.data.status_bars.len() > 1 && idx < file.data.status_bars.len() {
                file.data.status_bars.remove(idx);
                selection.clear();
            }
        }
        LayerAction::PasteStatusBars(bars) => {
            selection.clear();
            for mut bar in bars {
                bar.reassign_all_uids();
                file.data.status_bars.push(bar);
                selection.insert(vec![file.data.status_bars.len() - 1]);
            }
        }
        _ => {}
    }
}
