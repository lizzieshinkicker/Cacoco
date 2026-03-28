use crate::document::actions::InterlevelAction;
use crate::models::interlevel::{
    InterlevelAnim, InterlevelDefFile, InterlevelLayer, InterlevelScreen,
};
use std::collections::HashSet;

pub fn execute_interlevel_action(
    file: &mut InterlevelDefFile,
    selection: &mut HashSet<Vec<usize>>,
    action: InterlevelAction,
) {
    match action {
        InterlevelAction::AddScreen => {
            file.screens.push(InterlevelScreen {
                name: format!("INTER{:02}", file.screens.len()),
                version: "1.0.0".to_string(),
                ..Default::default()
            });
        }
        InterlevelAction::DuplicateScreen(idx) => {
            if let Some(screen) = file.screens.get(idx).cloned() {
                file.screens.insert(idx + 1, screen);
            }
        }
        InterlevelAction::DeleteScreen(idx) => {
            if file.screens.len() > 1 && idx < file.screens.len() {
                file.screens.remove(idx);
            }
        }
        InterlevelAction::RenameScreen(idx, new_name) => {
            if let Some(screen) = file.screens.get_mut(idx) {
                screen.name = new_name.to_uppercase();
            }
        }
        InterlevelAction::AddLayer { screen_idx } => {
            if let Some(screen) = file.screens.get_mut(screen_idx) {
                screen.data.layers.push(InterlevelLayer::default());
                selection.clear();
                selection.insert(vec![screen.data.layers.len() - 1]);
            }
        }
        InterlevelAction::AddAnim {
            screen_idx,
            layer_idx,
        } => {
            if let Some(screen) = file.screens.get_mut(screen_idx) {
                if let Some(layer) = screen.data.layers.get_mut(layer_idx) {
                    layer.anims.push(InterlevelAnim::default());
                    selection.clear();
                    selection.insert(vec![layer_idx, layer.anims.len() - 1]);
                }
            }
        }
        InterlevelAction::Delete { screen_idx, paths } => {
            if let Some(screen) = file.screens.get_mut(screen_idx) {
                let mut sorted_paths = paths.clone();
                sorted_paths.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| b.cmp(a)));
                for path in sorted_paths {
                    match path.as_slice() {
                        [l] => {
                            if *l < screen.data.layers.len() {
                                screen.data.layers.remove(*l);
                            }
                        }
                        [l, a] => {
                            if let Some(layer) = screen.data.layers.get_mut(*l) {
                                if *a < layer.anims.len() {
                                    layer.anims.remove(*a);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                selection.clear();
            }
        }
        InterlevelAction::MoveUp { screen_idx, path } => {
            if let Some(screen) = file.screens.get_mut(screen_idx) {
                match path.as_slice() {
                    [l] if *l > 0 => {
                        screen.data.layers.swap(*l, l - 1);
                        selection.clear();
                        selection.insert(vec![l - 1]);
                    }
                    [l, a] if *a > 0 => {
                        if let Some(layer) = screen.data.layers.get_mut(*l) {
                            layer.anims.swap(*a, a - 1);
                            selection.clear();
                            selection.insert(vec![*l, a - 1]);
                        }
                    }
                    _ => {}
                }
            }
        }
        InterlevelAction::MoveDown { screen_idx, path } => {
            if let Some(screen) = file.screens.get_mut(screen_idx) {
                match path.as_slice() {
                    [l] if *l + 1 < screen.data.layers.len() => {
                        screen.data.layers.swap(*l, l + 1);
                        selection.clear();
                        selection.insert(vec![l + 1]);
                    }
                    [l, a] => {
                        if let Some(layer) = screen.data.layers.get_mut(*l) {
                            if *a + 1 < layer.anims.len() {
                                layer.anims.swap(*a, a + 1);
                                selection.clear();
                                selection.insert(vec![*l, a + 1]);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        InterlevelAction::Duplicate { screen_idx, path } => {
            if let Some(screen) = file.screens.get_mut(screen_idx) {
                match path.as_slice() {
                    [l] => {
                        if let Some(layer) = screen.data.layers.get(*l).cloned() {
                            screen.data.layers.insert(l + 1, layer);
                        }
                    }
                    [l, a] => {
                        if let Some(layer) = screen.data.layers.get_mut(*l) {
                            if let Some(anim) = layer.anims.get(*a).cloned() {
                                layer.anims.insert(a + 1, anim);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
