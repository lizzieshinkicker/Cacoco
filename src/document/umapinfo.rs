use crate::document::actions::UmapAction;
use crate::models::umapinfo::{MapEntry, UmapField, UmapInfoFile};
use std::collections::HashSet;

/// Finds a map index by name (case-insensitive)
fn find_map_index(file: &UmapInfoFile, name: &str) -> Option<usize> {
    let upper_name = name.to_uppercase();
    file.data
        .maps
        .iter()
        .position(|m| m.mapname.to_uppercase() == upper_name)
}

/// Gets the field key for exit-type fields (next/nextsecret)
/// Returns `Some("next")` or `Some("nextsecret")`, or `None` for other field types
fn get_exit_field_key(field: &UmapField) -> Option<&'static str> {
    match field {
        UmapField::Next(_) => Some("next"),
        UmapField::NextSecret(_) => Some("nextsecret"),
        _ => None,
    }
}

/// Gets the field key for a UmapField, returning the key string if it's an exit field
fn get_field_key(field: &UmapField) -> Option<&'static str> {
    match field {
        UmapField::Next(_) => Some("next"),
        UmapField::NextSecret(_) => Some("nextsecret"),
        _ => None,
    }
}

/// Sets or updates a specific field in a map entry, removing existing instances first
fn set_field(map: &mut MapEntry, new_field: UmapField) {
    let Some(field_key) = get_exit_field_key(&new_field) else {
        return;
    };

    map.fields.retain(|f| {
        if let Some(f_key) = get_field_key(f) {
            f_key != field_key
        } else {
            true
        }
    });

    map.fields.push(new_field);
}

/// Removes a specific field type from a map entry
fn clear_field(map: &mut MapEntry, field_name: &str) {
    map.fields.retain(|f| {
        if let Some(f_key) = get_field_key(f) {
            f_key != field_name
        } else {
            true
        }
    });
}

/// Processes high-level mutations for UMAPINFO projects.
pub fn execute_umapinfo_action(
    file: &mut UmapInfoFile,
    action: UmapAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        UmapAction::AddMap { x, y } => {
            let next_num = file.data.maps.len() + 1;
            file.data.maps.push(MapEntry {
                mapname: format!("MAP{:02}", next_num),
                fields: vec![UmapField::LevelName("New Level".to_string())],
            });
            let new_idx = file.data.maps.len() - 1;
            selection.clear();
            selection.insert(vec![new_idx]);

            if x != 0.0 || y != 0.0 {
                file.set_node_pos(&format!("MAP{:02}", next_num), x, y);
            }
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
            let new_positions =
                crate::models::umap_graph::UmapGraph::generate_topological_layout(file);
            for (map_id, (x, y)) in new_positions {
                file.set_node_pos(&map_id, x, y);
            }
        }
        UmapAction::SetNormalExit { map_name, target } => {
            if let Some(idx) = find_map_index(file, &map_name) {
                let target_upper = target.to_uppercase();
                if file
                    .data
                    .maps
                    .iter()
                    .any(|m| m.mapname.to_uppercase() == target_upper)
                {
                    set_field(&mut file.data.maps[idx], UmapField::Next(target_upper));
                }
            }
        }
        UmapAction::SetSecretExit { map_name, target } => {
            if let Some(idx) = find_map_index(file, &map_name) {
                let target_upper = target.to_uppercase();
                if file
                    .data
                    .maps
                    .iter()
                    .any(|m| m.mapname.to_uppercase() == target_upper)
                {
                    set_field(
                        &mut file.data.maps[idx],
                        UmapField::NextSecret(target_upper),
                    );
                }
            }
        }
        UmapAction::ClearNormalExit(map_name) => {
            if let Some(idx) = find_map_index(file, &map_name) {
                clear_field(&mut file.data.maps[idx], "next");
            }
        }
        UmapAction::ClearSecretExit(map_name) => {
            if let Some(idx) = find_map_index(file, &map_name) {
                clear_field(&mut file.data.maps[idx], "nextsecret");
            }
        }
    }
}
