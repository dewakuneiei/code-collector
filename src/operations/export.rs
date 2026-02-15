use std::fs;
use std::path::Path;
use crate::models::dir_node::DirNode;

pub const DEFAULT_OUTPUT_FILENAME: &str = "full_code.txt";

pub fn calculate_stats(dir: &DirNode) -> (u64, usize) {
    let mut size = 0;
    let mut files = 0;
    for file in &dir.children_files {
        if file.selected {
            size += file.size_bytes;
            files += 1;
        }
    }
    for sub in &dir.children_dirs {
        let (s, f) = calculate_stats(sub);
        size += s;
        files += f;
    }
    (size, files)
}

pub fn collect_content_string(dir: &DirNode, content: &mut String) {
    for file in &dir.children_files {
        if file.selected {
            if let Ok(code) = fs::read_to_string(&file.path) {
                 content.push_str(&format!(
                    "\n\n{}\nFILE: {}\nLANGUAGE: {}\n{}\n\n",
                    "=".repeat(50), file.rel_path, file.extension, "=".repeat(50)
                ));
                content.push_str(&code);
            }
        }
    }
    for sub in &dir.children_dirs {
        collect_content_string(sub, content);
    }
}

pub fn write_files_recursive(dir: &DirNode, base_target_path: &Path) -> std::io::Result<()> {
    for file in &dir.children_files {
        if file.selected {
            let target_file_path = base_target_path.join(&file.rel_path);
            if let Some(parent) = target_file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&file.path, target_file_path)?;
        }
    }
    for sub_dir in &dir.children_dirs {
        write_files_recursive(sub_dir, base_target_path)?;
    }
    Ok(())
}