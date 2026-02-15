use std::path::PathBuf;
use super::file_node::FileNode;

#[derive(Clone)]
pub struct DirNode {
    pub name: String,
    pub path: PathBuf,
    pub children_dirs: Vec<DirNode>,
    pub children_files: Vec<FileNode>,
}