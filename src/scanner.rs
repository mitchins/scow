use crate::app::FileNode;
use jwalk::WalkDir;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use anyhow::Result;

pub struct Scanner;

impl Scanner {
    pub fn scan<P: AsRef<Path>>(path: P) -> Result<FileNode> {
        let path = path.as_ref();
        let mut nodes: HashMap<PathBuf, FileNode> = HashMap::new();
        
        // First pass: collect all entries
        for entry in WalkDir::new(path).sort(true) {
            let entry = entry?;
            let entry_path = entry.path();
            let metadata = entry.metadata()?;
            
            let size = if metadata.is_file() {
                metadata.len()
            } else {
                0
            };

            let node = FileNode {
                path: entry_path.to_path_buf(),
                size,
                is_dir: metadata.is_dir(),
                children: Vec::new(),
                modified: metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            };

            nodes.insert(entry_path.to_path_buf(), node);
        }

        // Second pass: build tree structure and calculate directory sizes
        build_tree(path, &mut nodes)
    }
}

fn build_tree(root_path: &Path, nodes: &mut HashMap<PathBuf, FileNode>) -> Result<FileNode> {
    let mut root = nodes
        .remove(root_path)
        .ok_or_else(|| anyhow::anyhow!("Root path not found"))?;

    let mut children = Vec::new();
    let mut total_size = root.size;

    // Find direct children
    for path in nodes.keys().cloned().collect::<Vec<_>>() {
        if let Some(parent) = path.parent() {
            if parent == root_path {
                children.push(path);
            }
        }
    }

    // Recursively build children
    for child_path in children {
        if let Ok(child_node) = build_tree(&child_path, nodes) {
            total_size += child_node.size;
            root.children.push(child_node);
        }
    }

    // Sort children by size (descending)
    root.children.sort_by(|a, b| b.size.cmp(&a.size));
    
    if root.is_dir {
        root.size = total_size;
    }

    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_scanner_basic() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create test structure
        fs::write(temp_path.join("file1.txt"), "content1")?;
        fs::write(temp_path.join("file2.txt"), "longer content")?;
        fs::create_dir(temp_path.join("subdir"))?;
        fs::write(temp_path.join("subdir/file3.txt"), "nested")?;

        let root = Scanner::scan(temp_path)?;

        assert_eq!(root.is_dir, true);
        assert!(root.children.len() >= 2);
        
        Ok(())
    }

    #[test]
    fn test_scanner_size_calculation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create files with known sizes
        fs::write(temp_path.join("small.txt"), "abc")?; // 3 bytes
        fs::write(temp_path.join("large.txt"), "x".repeat(100))?; // 100 bytes

        let root = Scanner::scan(temp_path)?;

        // Root should include all file sizes
        assert!(root.size >= 103);
        
        Ok(())
    }

    #[test]
    fn test_scanner_sorting() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        fs::write(temp_path.join("small.txt"), "a")?;
        fs::write(temp_path.join("large.txt"), "x".repeat(100))?;

        let root = Scanner::scan(temp_path)?;

        // Children should be sorted by size descending
        if root.children.len() >= 2 {
            assert!(root.children[0].size >= root.children[1].size);
        }
        
        Ok(())
    }
}
