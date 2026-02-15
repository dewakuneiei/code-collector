use std::path::PathBuf;

#[derive(Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub rel_path: String,
    pub extension: String,
    pub selected: bool,
    pub size_bytes: u64,
}