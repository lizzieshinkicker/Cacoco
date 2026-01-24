use crate::document::LayerAction;
use crate::models::skydefs::{SkyDef, SkyDefsFile};
use std::collections::HashSet;

pub fn execute_sky_action(
    file: &mut SkyDefsFile,
    action: LayerAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        LayerAction::AddSky => {
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
        LayerAction::DeleteSky(idx) => {
            if idx < file.data.skies.len() {
                file.data.skies.remove(idx);
                selection.clear();
            }
        }
        _ => {}
    }
}
