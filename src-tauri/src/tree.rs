use serde::Serialize;

/// A node in the scanned filesystem tree.
/// Sent to frontend as JSON after scan completes.
#[derive(Serialize, Clone, Debug)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
    pub file_count: u32,
    pub dir_count: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<FileNode>,
    pub modified: Option<u64>,
    pub extension: Option<String>,
    /// Depth from scan root (0 = root)
    pub depth: u32,
}

/// Lightweight progress event streamed to frontend during scan.
#[derive(Serialize, Clone, Debug)]
pub struct ScanProgress {
    pub scanned_files: u64,
    pub scanned_bytes: u64,
    pub current_path: String,
}

/// Volume / drive descriptor.
#[derive(Serialize, Clone, Debug)]
pub struct Volume {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub file_system: String,
    pub is_removable: bool,
}
