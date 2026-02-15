use crate::models::dir_node::DirNode;

pub fn set_dir_selection(dir: &mut DirNode, state: bool, query: &str) {
    let is_visible = |name: &str| query.is_empty() || name.to_lowercase().contains(&query.to_lowercase());

    for file in &mut dir.children_files {
        if is_visible(&file.name) {
            file.selected = state;
        }
    }
    for sub in &mut dir.children_dirs {
        set_dir_selection(sub, state, query);
    }
}

pub fn is_dir_fully_selected(dir: &DirNode) -> bool {
    if dir.children_files.is_empty() && dir.children_dirs.is_empty() { return false; }
    for file in &dir.children_files {
        if !file.selected { return false; }
    }
    for sub in &dir.children_dirs {
        if !is_dir_fully_selected(sub) { return false; }
    }
    true
}