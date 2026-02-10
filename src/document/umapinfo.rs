use crate::document::LayerAction;
use crate::models::umapinfo::{MapEntry, UmapField, UmapInfoFile};
use std::collections::HashSet;

/// Processes high-level mutations for UMAPINFO projects.
pub fn execute_umapinfo_action(
    file: &mut UmapInfoFile,
    action: LayerAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        LayerAction::AddMap => {
            let next_num = file.data.maps.len() + 1;
            file.data.maps.push(MapEntry {
                mapname: format!("MAP{:02}", next_num),
                fields: vec![UmapField::LevelName("New Level".to_string())],
            });
            let new_idx = file.data.maps.len() - 1;
            selection.clear();
            selection.insert(vec![new_idx]);
        }
        LayerAction::DeleteMap(idx) => {
            if idx < file.data.maps.len() {
                file.data.maps.remove(idx);
                selection.clear();
            }
        }
        _ => {}
    }
}
