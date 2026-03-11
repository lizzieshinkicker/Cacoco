use crate::models::sbardef::{ElementWrapper, StatusBarLayout};

#[derive(Debug, Clone)]
pub enum DocumentAction {
    /// Forces an undo snapshot.
    UndoSnapshot,
    /// Actions shared by any hierarchical tree (SBARDEF elements, INTERLEVEL anims).
    Tree(TreeAction),
    /// Specialized SBARDEF layout actions.
    SBar(SBarAction),
    /// Specialized SKYDEFS actions.
    Sky(SkyAction),
    /// Specialized UMAPINFO actions.
    Umap(UmapAction),
}

#[derive(Debug, Clone)]
pub enum TreeAction {
    Delete(Vec<Vec<usize>>),
    Duplicate(Vec<Vec<usize>>),
    MoveUp(Vec<usize>),
    MoveDown(Vec<usize>),
    MoveSelection {
        sources: Vec<Vec<usize>>,
        target_parent: Vec<usize>,
        insert_idx: usize,
    },
    Paste {
        parent_path: Vec<usize>,
        insert_idx: usize,
        elements: Vec<ElementWrapper>,
    },
    Translate {
        paths: Vec<Vec<usize>>,
        dx: i32,
        dy: i32,
    },
    Select(Vec<Vec<usize>>),
    ToggleSelection(Vec<Vec<usize>>),
    Add {
        parent_path: Vec<usize>,
        insert_idx: usize,
        element: ElementWrapper,
    },
    Group(Vec<Vec<usize>>),
}

#[derive(Debug, Clone)]
pub enum SBarAction {
    AddStatusBar,
    DuplicateStatusBar(usize),
    MoveStatusBar { source: usize, target: usize },
    DeleteStatusBar(usize),
    PasteStatusBars(Vec<StatusBarLayout>),
}

#[derive(Debug, Clone)]
pub enum SkyAction {
    Add,
    Delete(usize),
    Move { source: usize, target: usize },
    Duplicate(usize),
}

#[derive(Debug, Clone)]
pub enum UmapAction {
    AddMap { x: f32, y: f32 },
    DeleteMap(usize),
    UpdateNodePos(String, f32, f32),
    ResetLayout,
}
