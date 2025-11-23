use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub size: u64,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub modified: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Delete,
    Move(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Staging {
    pub ops: HashMap<PathBuf, Action>,
    pub net_size_change: i64,
}

impl Staging {
    pub fn new() -> Self {
        Self {
            ops: HashMap::new(),
            net_size_change: 0,
        }
    }

    pub fn toggle_delete(&mut self, path: PathBuf, size: u64) {
        if self.ops.contains_key(&path) {
            // Un-staging a delete: we're no longer freeing this space
            self.ops.remove(&path);
            self.net_size_change += size as i64;  // Less space will be freed
        } else {
            // Staging a delete: we will free this space
            self.ops.insert(path, Action::Delete);
            self.net_size_change -= size as i64;  // More space will be freed
        }
    }

    pub fn set_move(&mut self, path: PathBuf, dest: PathBuf) {
        self.ops.insert(path, Action::Move(dest));
    }

    pub fn clear(&mut self) {
        self.ops.clear();
        self.net_size_change = 0;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Scanning,
    Browsing,
    DestinationPicker,
    Confirming,
    Processing,
}

pub struct AppState {
    pub current_path: PathBuf,
    pub root_node: Option<FileNode>,
    pub selection_index: usize,
    pub staging: Staging,
    pub mode: AppMode,
    pub config: AppConfig,
    pub should_quit: bool,
}

impl AppState {
    pub fn new(path: PathBuf, config: AppConfig) -> Self {
        Self {
            current_path: path,
            root_node: None,
            selection_index: 0,
            staging: Staging::new(),
            mode: AppMode::Browsing,
            config,
            should_quit: false,
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub destinations: Vec<Destination>,
}

#[derive(Debug, Clone)]
pub struct Destination {
    pub name: String,
    pub path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            destinations: vec![
                Destination {
                    name: "Home".to_string(),
                    path: "~".to_string(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staging_toggle_delete() {
        let mut staging = Staging::new();
        let path = PathBuf::from("/test/file.txt");
        let size = 1000;

        // Mark for delete
        staging.toggle_delete(path.clone(), size);
        assert_eq!(staging.ops.get(&path), Some(&Action::Delete));
        assert_eq!(staging.net_size_change, -1000);

        // Toggle back (unmark)
        staging.toggle_delete(path.clone(), size);
        assert_eq!(staging.ops.get(&path), None);
        assert_eq!(staging.net_size_change, 0);
    }

    #[test]
    fn test_staging_stats_calculation() {
        let mut staging = Staging::new();

        // Stage 3 files for deletion
        staging.toggle_delete(PathBuf::from("/file1.txt"), 100_000_000);
        staging.toggle_delete(PathBuf::from("/file2.txt"), 100_000_000);
        staging.toggle_delete(PathBuf::from("/file3.txt"), 100_000_000);

        assert_eq!(staging.net_size_change, -300_000_000);
        assert_eq!(staging.ops.len(), 3);
    }

    #[test]
    fn test_staging_set_move() {
        let mut staging = Staging::new();
        let source = PathBuf::from("/source/file.txt");
        let dest = PathBuf::from("/dest");

        staging.set_move(source.clone(), dest.clone());
        assert_eq!(staging.ops.get(&source), Some(&Action::Move(dest)));
    }
}
