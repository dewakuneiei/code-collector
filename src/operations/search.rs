use crate::models::dir_node::DirNode;

pub fn matches_search(dir: &DirNode, query: &str) -> bool {
    if query.is_empty() { return true; }
    let q = query.to_lowercase();
    
    for file in &dir.children_files {
        if file.name.to_lowercase().contains(&q) { return true; }
    }

    for sub in &dir.children_dirs {
        if matches_search(sub, query) { return true; }
    }
    
    false
}