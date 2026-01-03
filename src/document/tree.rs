use crate::document::LayerAction;
use crate::model::{CanvasDef, CommonAttrs, Element, ElementWrapper, SBarDefFile};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Processes actions that target the hierarchical element tree.
///
/// This dispatcher implements the Command Pattern, delegating tree
/// mutations to specialized private operation handlers.
pub fn execute_tree_action(
    file: &mut SBarDefFile,
    action: LayerAction,
    selection: &mut HashSet<Vec<usize>>,
) {
    match action {
        LayerAction::DeleteSelection(paths) => op_delete(file, selection, paths),
        LayerAction::DuplicateSelection(paths) => op_duplicate(file, selection, paths),
        LayerAction::MoveUp(path) => op_move_up(file, selection, path),
        LayerAction::MoveDown(path) => op_move_down(file, selection, path),
        LayerAction::MoveSelection {
            sources,
            target_parent,
            insert_idx,
        } => op_move_selection(file, selection, sources, target_parent, insert_idx),
        LayerAction::Add {
            parent_path,
            insert_idx,
            element,
        } => op_add(file, selection, parent_path, insert_idx, element),
        LayerAction::Paste {
            parent_path,
            insert_idx,
            elements,
        } => op_paste(file, selection, parent_path, insert_idx, elements),
        LayerAction::TranslateSelection { paths, dx, dy } => op_translate(file, paths, dx, dy),
        LayerAction::GroupSelection(paths) => op_group(file, selection, paths),
        _ => {}
    }
}

fn op_delete(file: &mut SBarDefFile, selection: &mut HashSet<Vec<usize>>, paths: Vec<Vec<usize>>) {
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

fn op_duplicate(
    file: &mut SBarDefFile,
    selection: &mut HashSet<Vec<usize>>,
    paths: Vec<Vec<usize>>,
) {
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
                    c.reassign_uids();
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

fn op_move_up(file: &mut SBarDefFile, selection: &mut HashSet<Vec<usize>>, path: Vec<usize>) {
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

fn op_move_down(file: &mut SBarDefFile, selection: &mut HashSet<Vec<usize>>, path: Vec<usize>) {
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

fn op_move_selection(
    file: &mut SBarDefFile,
    selection: &mut HashSet<Vec<usize>>,
    sources: Vec<Vec<usize>>,
    target_parent: Vec<usize>,
    insert_idx: usize,
) {
    execute_move_selection(file, sources, target_parent, insert_idx, selection);
}

fn op_add(
    file: &mut SBarDefFile,
    selection: &mut HashSet<Vec<usize>>,
    parent_path: Vec<usize>,
    insert_idx: usize,
    element: ElementWrapper,
) {
    insert_element_and_select(file, parent_path, insert_idx, element, selection);
}

fn op_paste(
    file: &mut SBarDefFile,
    selection: &mut HashSet<Vec<usize>>,
    parent_path: Vec<usize>,
    insert_idx: usize,
    elements: Vec<ElementWrapper>,
) {
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

fn op_translate(file: &mut SBarDefFile, paths: Vec<Vec<usize>>, dx: i32, dy: i32) {
    for path in paths {
        if let Some(element) = file.get_element_mut(&path) {
            let common = element.get_common_mut();
            common.x += dx;
            common.y += dy;
        }
    }
}

fn op_group(file: &mut SBarDefFile, selection: &mut HashSet<Vec<usize>>, paths: Vec<Vec<usize>>) {
    let filtered_sources = filter_to_roots(paths);
    if filtered_sources.is_empty() {
        return;
    }

    let mut sorted_for_pos = filtered_sources.clone();
    sorted_for_pos.sort();

    let first_path = &sorted_for_pos[0];
    if first_path.len() < 2 {
        return;
    }

    let target_parent = first_path[..first_path.len() - 1].to_vec();
    let mut insert_idx = first_path[first_path.len() - 1];

    let mut to_remove = filtered_sources.clone();
    sort_paths_for_removal(&mut to_remove);

    let mut moved_elements = Vec::new();
    for src in to_remove {
        let src_parent = &src[0..src.len() - 1];
        let src_idx = *src.last().unwrap();
        if src_parent == target_parent && src_idx < insert_idx {
            insert_idx -= 1;
        }
        if let Some((list, idx)) = get_parent_list_and_idx(file, &src) {
            moved_elements.push(list.remove(idx));
        }
    }

    moved_elements.reverse();
    let new_canvas = ElementWrapper {
        data: Element::Canvas(CanvasDef {
            common: CommonAttrs {
                children: moved_elements,
                ..Default::default()
            },
        }),
        ..Default::default()
    };

    insert_element_and_select(file, target_parent, insert_idx, new_canvas, selection);
}

fn execute_move_selection(
    file: &mut SBarDefFile,
    sources: Vec<Vec<usize>>,
    mut target_parent: Vec<usize>,
    mut insert_idx: usize,
    selection: &mut HashSet<Vec<usize>>,
) {
    let filtered_sources = filter_to_roots(sources);

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

/// Helper to determine the best place to insert a new element based on the current selection.
pub fn determine_insertion_point(
    file: &SBarDefFile,
    selection: &HashSet<Vec<usize>>,
    current_bar_idx: usize,
) -> (Vec<usize>, usize) {
    if selection.len() == 1 {
        let path = selection.iter().next().unwrap();

        if path.len() > 1 {
            let parent_path = path[0..path.len() - 1].to_vec();
            let selected_idx = *path.last().unwrap();

            return (parent_path, selected_idx);
        }
    }

    if let Some(bar) = file.data.status_bars.get(current_bar_idx) {
        (vec![current_bar_idx], bar.children.len())
    } else {
        (vec![current_bar_idx], 0)
    }
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

fn filter_to_roots(paths: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut roots = paths;
    roots.sort_by_key(|p| p.len());
    let mut filtered = Vec::new();
    for p in roots {
        if !filtered.iter().any(|f: &Vec<usize>| p.starts_with(f)) {
            filtered.push(p);
        }
    }
    filtered
}

fn insert_element_and_select(
    file: &mut SBarDefFile,
    parent_path: Vec<usize>,
    insert_idx: usize,
    element: ElementWrapper,
    selection: &mut HashSet<Vec<usize>>,
) {
    if let Some(list) = get_child_list_mut(file, &parent_path) {
        let actual_idx = insert_idx.min(list.len());
        list.insert(actual_idx, element);
        selection.clear();
        let mut new_path = parent_path;
        new_path.push(actual_idx);
        selection.insert(new_path);
    }
}
