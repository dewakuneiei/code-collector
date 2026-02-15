pub mod thread;

use crate::models::dir_node::DirNode;

pub enum ScanMessage {
    Progress(usize),       // "I found X files so far"
    Finished(DirNode),     // "Here is the completed tree"
    Cancelled,             // "User stopped me"
}