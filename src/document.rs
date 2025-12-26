use crate::model::{ElementWrapper, SBarDefFile, new_uid};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum LayerAction {
    UndoSnapshot,
    DeleteSelection(Vec<Vec<usize>>),
    DuplicateSelection(Vec<Vec<usize>>),
    MoveUp(Vec<usize>),
    MoveDown(Vec<usize>),
    MoveSelection {
        sources: Vec<Vec<usize>>,
        target_parent: Vec<usize>,
        insert_idx: usize,
    },
    Add {
        parent_path: Vec<usize>,
        insert_idx: usize,
        element: ElementWrapper,
    },
    Paste {
        parent_path: Vec<usize>,
        insert_idx: usize,
        elements: Vec<ElementWrapper>,
    },
    TranslateSelection {
        paths: Vec<Vec<usize>>,
        dx: i32,
        dy: i32,
    },
}

pub fn execute_layer_action(
    file: &mut SBarDefFile,
    action: LayerAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        LayerAction::UndoSnapshot => {}
        LayerAction::DeleteSelection(paths) => {
            let mut sorted_paths = paths.clone();
            sort_paths_for_removal(&mut sorted_paths);
            selection.clear();

            for path in sorted_paths {
                if let Some((list, idx)) = get_parent_list_and_idx(file, &path) {
                    if idx < list.len() {
                        list.remove(idx);
                    }
                }
            }
        }
        LayerAction::DuplicateSelection(paths) => {
            let mut by_parent: HashMap<Vec<usize>, Vec<usize>> = HashMap::new();
            for path in paths {
                if path.len() >= 2 {
                    let parent = path[0..path.len() - 1].to_vec();
                    let idx = *path.last().unwrap();
                    by_parent.entry(parent).or_default().push(idx);
                }
            }

            selection.clear();
            for (parent_path, mut indices) in by_parent {
                indices.sort();
                let first_idx = indices[0];

                if let Some(list) = get_child_list_mut(file, &parent_path) {
                    let mut clones = Vec::new();
                    for &idx in &indices {
                        if idx < list.len() {
                            let mut c = list[idx].clone();
                            c.uid = new_uid();
                            clones.push(c);
                        }
                    }

                    let mut current_insert = first_idx;
                    for clone in clones {
                        if current_insert <= list.len() {
                            list.insert(current_insert, clone);
                            let mut new_path = parent_path.clone();
                            new_path.push(current_insert);
                            selection.insert(new_path);
                            current_insert += 1;
                        }
                    }
                }
            }
        }
        LayerAction::MoveUp(path) => {
            if let Some((list, idx)) = get_parent_list_and_idx(file, &path) {
                if idx > 0 {
                    list.swap(idx, idx - 1);
                    selection.clear();
                    let mut new_path = path[0..path.len() - 1].to_vec();
                    new_path.push(idx - 1);
                    selection.insert(new_path);
                }
            }
        }
        LayerAction::MoveDown(path) => {
            if let Some((list, idx)) = get_parent_list_and_idx(file, &path) {
                if idx < list.len() - 1 {
                    list.swap(idx, idx + 1);
                    selection.clear();
                    let mut new_path = path[0..path.len() - 1].to_vec();
                    new_path.push(idx + 1);
                    selection.insert(new_path);
                }
            }
        }
        LayerAction::MoveSelection {
            sources,
            target_parent,
            insert_idx,
        } => {
            execute_move_selection(file, sources, target_parent, insert_idx, selection);
        }
        LayerAction::Add {
            parent_path,
            insert_idx,
            element,
        } => {
            if let Some(list) = get_child_list_mut(file, &parent_path) {
                let actual_idx = insert_idx.min(list.len());
                list.insert(actual_idx, element);
                selection.clear();
                let mut new_sel_path = parent_path;
                new_sel_path.push(actual_idx);
                selection.insert(new_sel_path);
            }
        }
        LayerAction::Paste {
            parent_path,
            insert_idx,
            elements,
        } => {
            if let Some(list) = get_child_list_mut(file, &parent_path) {
                selection.clear();
                let mut current_pos = insert_idx.min(list.len());
                for el in elements {
                    list.insert(current_pos, el);
                    let mut new_path = parent_path.clone();
                    new_path.push(current_pos);
                    selection.insert(new_path);
                    current_pos += 1;
                }
            }
        }
        LayerAction::TranslateSelection { paths, dx, dy } => {
            for path in paths {
                if let Some(element) = file.get_element_mut(&path) {
                    let common = element.get_common_mut();
                    common.x += dx;
                    common.y += dy;
                }
            }
        }
    }
}

fn execute_move_selection(
    file: &mut SBarDefFile,
    sources: Vec<Vec<usize>>,
    mut target_parent: Vec<usize>,
    mut insert_idx: usize,
    selection: &mut HashSet<Vec<usize>>,
) {
    let mut roots = sources.clone();
    roots.sort_by_key(|p| p.len());
    let mut filtered_sources = Vec::new();
    for p in roots {
        if !filtered_sources
            .iter()
            .any(|f: &Vec<usize>| p.starts_with(f))
        {
            filtered_sources.push(p);
        }
    }

    for src in &filtered_sources {
        if target_parent.starts_with(src) {
            return;
        }
    }

    let mut to_remove = filtered_sources.clone();
    sort_paths_for_removal(&mut to_remove);

    let mut moved_elements = Vec::new();
    for src in to_remove {
        let src_parent = &src[0..src.len() - 1];
        let src_idx = *src.last().unwrap();

        if target_parent.starts_with(src_parent) && src_parent.len() < target_parent.len() {
            let depth = src_parent.len();
            if src_idx < target_parent[depth] {
                target_parent[depth] -= 1;
            }
        }

        if src_parent == target_parent && src_idx < insert_idx {
            insert_idx -= 1;
        }

        if let Some((list, idx)) = get_parent_list_and_idx(file, &src) {
            if idx < list.len() {
                moved_elements.push(list.remove(idx));
            }
        }
    }

    moved_elements.reverse();

    if let Some(target_list) = get_child_list_mut(file, &target_parent) {
        let safe_idx = insert_idx.min(target_list.len());
        selection.clear();
        for (i, elem) in moved_elements.into_iter().enumerate() {
            let final_idx = safe_idx + i;
            target_list.insert(final_idx, elem);

            let mut new_path = target_parent.clone();
            new_path.push(final_idx);
            selection.insert(new_path);
        }
    }
}

pub fn determine_insertion_point(
    selection: &HashSet<Vec<usize>>,
    current_bar_idx: usize,
) -> (Vec<usize>, usize) {
    if selection.len() == 1 {
        let path = selection.iter().next().unwrap();
        if path.len() > 1 {
            let parent_path = path[0..path.len() - 1].to_vec();
            let selected_idx = *path.last().unwrap();
            return (parent_path, selected_idx + 1);
        }
    }
    (vec![current_bar_idx], 0)
}

fn sort_paths_for_removal(paths: &mut Vec<Vec<usize>>) {
    paths.sort_by(|a, b| match b.len().cmp(&a.len()) {
        Ordering::Equal => b.cmp(a),
        ord => ord,
    });
}

fn get_child_list_mut<'a>(
    file: &'a mut SBarDefFile,
    parent_path: &[usize],
) -> Option<&'a mut Vec<ElementWrapper>> {
    if parent_path.is_empty() {
        return None;
    }
    let bar_idx = parent_path[0];
    if bar_idx >= file.data.status_bars.len() {
        return None;
    }

    let bar = &mut file.data.status_bars[bar_idx];
    if parent_path.len() == 1 {
        return Some(&mut bar.children);
    }

    let mut current_element = bar.children.get_mut(parent_path[1])?;
    for &child_idx in &parent_path[2..] {
        current_element = current_element
            .get_common_mut()
            .children
            .get_mut(child_idx)?;
    }
    Some(&mut current_element.get_common_mut().children)
}

fn get_parent_list_and_idx<'a>(
    file: &'a mut SBarDefFile,
    path: &[usize],
) -> Option<(&'a mut Vec<ElementWrapper>, usize)> {
    if path.len() < 2 {
        return None;
    }
    let parent_path = &path[0..path.len() - 1];
    let target_idx = *path.last().unwrap();
    get_child_list_mut(file, parent_path).map(|list| (list, target_idx))
}
