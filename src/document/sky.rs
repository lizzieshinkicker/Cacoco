use crate::document::actions::SkyAction;
use crate::models::skydefs::{SkyDef, SkyDefsFile};
use std::collections::HashSet;

pub fn execute_sky_action(
    file: &mut SkyDefsFile,
    action: SkyAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        SkyAction::Add => {
            file.data.skies.push(SkyDef {
                name: "SKY1".to_string(),
                scalex: 1.0,
                scaley: 1.0,
                mid: 100.0,
                ..Default::default()
            });
            let new_idx = file.data.skies.len() - 1;
            selection.clear();
            selection.insert(vec![new_idx]);
        }
        SkyAction::Delete(idx) => {
            if idx < file.data.skies.len() {
                file.data.skies.remove(idx);
                selection.clear();
            }
        }
        SkyAction::Move { source, target } => {
            if source < file.data.skies.len() {
                let element = file.data.skies.remove(source);
                let mut final_target = target;
                if source < target {
                    final_target = final_target.saturating_sub(1);
                }
                let safe_target = final_target.min(file.data.skies.len());
                file.data.skies.insert(safe_target, element);
                selection.clear();
                selection.insert(vec![safe_target]);
            }
        }
        SkyAction::Duplicate(idx) => {
            if let Some(sky) = file.data.skies.get(idx) {
                let mut new_sky = sky.clone();
                new_sky.name = format!("{} (Copy)", new_sky.name);
                file.data.skies.insert(idx + 1, new_sky);
                selection.clear();
                selection.insert(vec![idx + 1]);
            }
        }
    }
}
