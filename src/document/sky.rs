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
    }
}
