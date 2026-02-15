use std::fs;
use std::path::Path;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::Sender;
use crate::models::{dir_node::DirNode, file_node::FileNode};
use super::ScanMessage;
use crate::operations::export::DEFAULT_OUTPUT_FILENAME;

const IGNORE_DIRS: &[&str] = &[
    ".git", ".vscode", "node_modules", "vendor", "__pycache__", 
    ".idea", "target", "dist", "build", "coverage", ".next", ".nuxt", "storage"
];

// Static helper for the thread
pub fn read_dir_recursive_threaded(
    dir_path: &Path, 
    root_path: &Path, 
    tx: &Sender<ScanMessage>, 
    cancel_flag: &Arc<AtomicBool>
) -> Option<DirNode> {
    
    // 1. Check Cancellation
    if cancel_flag.load(Ordering::Relaxed) {
        return None;
    }

    let mut node = DirNode {
        name: dir_path.file_name().unwrap_or_default().to_string_lossy().to_string(),
        path: dir_path.to_path_buf(),
        children_dirs: Vec::new(),
        children_files: Vec::new(),
    };

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            // Check cancellation inside the loop for faster response
            if cancel_flag.load(Ordering::Relaxed) { return None; }

            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') && name != ".env" { continue; }
            if IGNORE_DIRS.contains(&name.as_str()) { continue; }

            if path.is_dir() {
                // Recurse
                if let Some(child) = read_dir_recursive_threaded(&path, root_path, tx, cancel_flag) {
                    node.children_dirs.push(child);
                }
            } else {
                let name_lower = name.to_lowercase();
                let extension = if name_lower.ends_with(".blade.php") {
                    "blade.php".to_string()
                } else {
                    path.extension()
                        .map(|e| e.to_string_lossy().to_string().to_lowercase())
                        .unwrap_or_default()
                };
                
                let rel_path = path.strip_prefix(root_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                if name == DEFAULT_OUTPUT_FILENAME { continue; }

                let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                node.children_files.push(FileNode {
                    name,
                    path,
                    rel_path,
                    extension,
                    selected: false,
                    size_bytes,
                });

                // NOTIFY UI: Found a file
                let _ = tx.send(ScanMessage::Progress(1));
            }
        }
    }

    // Sort for clean display
    node.children_dirs.sort_by(|a, b| a.name.cmp(&b.name));
    node.children_files.sort_by(|a, b| a.name.cmp(&b.name));
    Some(node)
}