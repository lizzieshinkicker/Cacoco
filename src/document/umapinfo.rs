use crate::document::actions::UmapAction;
use crate::models::umapinfo::{MapEntry, UmapField, UmapInfoFile};
use std::collections::HashSet;

/// Processes high-level mutations for UMAPINFO projects.
pub fn execute_umapinfo_action(
    file: &mut UmapInfoFile,
    action: UmapAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        UmapAction::AddMap => {
            let next_num = file.data.maps.len() + 1;
            file.data.maps.push(MapEntry {
                mapname: format!("MAP{:02}", next_num),
                fields: vec![UmapField::LevelName("New Level".to_string())],
            });
            let new_idx = file.data.maps.len() - 1;
            selection.clear();
            selection.insert(vec![new_idx]);
        }
        UmapAction::DeleteMap(idx) => {
            if idx < file.data.maps.len() {
                file.data.maps.remove(idx);
                selection.clear();
            }
        }
        UmapAction::UpdateNodePos(name, x, y) => {
            file.set_node_pos(&name, x, y);
        }
        UmapAction::ResetLayout => {
            if let Some(obj) = file.metadata.as_object_mut() {
                obj.remove("node_positions");
            }
        }
    }
}
