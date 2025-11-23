use crate::app::Action;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use anyhow::Result;

pub struct Worker;

impl Worker {
    pub fn execute(operations: Vec<(PathBuf, Action)>) -> Result<()> {
        for (path, action) in operations {
            match action {
                Action::Delete => {
                    Self::execute_delete(&path)?;
                }
                Action::Move(dest) => {
                    Self::execute_move(&path, &dest)?;
                }
            }
        }
        Ok(())
    }

    fn execute_delete(path: &Path) -> Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn execute_move(source: &Path, dest: &Path) -> Result<()> {
        // Check if destination looks like a remote path
        if let Some(dest_str) = dest.to_str() {
            if dest_str.starts_with("ssh:") || dest_str.contains('@') {
                return Self::execute_remote_move(source, dest_str);
            }
        }

        // Local move
        if dest.is_dir() {
            // Move into directory
            if let Some(filename) = source.file_name() {
                let target = dest.join(filename);
                fs::rename(source, target)?;
            } else {
                anyhow::bail!("Source path has no filename component");
            }
        } else {
            // Move to specific path
            fs::rename(source, dest)?;
        }
        Ok(())
    }

    fn execute_remote_move(source: &Path, dest: &str) -> Result<()> {
        // Validate destination format
        let dest_clean = dest.strip_prefix("ssh://").unwrap_or(dest);
        
        // Basic validation to prevent command injection
        if dest_clean.contains(';') || dest_clean.contains('|') || dest_clean.contains('&') {
            anyhow::bail!("Invalid destination path: potentially unsafe characters");
        }
        
        let output = Command::new("rsync")
            .arg("-avz")
            .arg("--progress")
            .arg(source)
            .arg(dest_clean)
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "rsync failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_execute_delete_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "content")?;

        Worker::execute(vec![(file_path.clone(), Action::Delete)])?;

        assert!(!file_path.exists());
        Ok(())
    }

    #[test]
    fn test_execute_delete_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let dir_path = temp_dir.path().join("testdir");
        fs::create_dir(&dir_path)?;
        fs::write(dir_path.join("file.txt"), "content")?;

        Worker::execute(vec![(dir_path.clone(), Action::Delete)])?;

        assert!(!dir_path.exists());
        Ok(())
    }

    #[test]
    fn test_execute_move_local() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let source = temp_dir.path().join("source.txt");
        let dest_dir = temp_dir.path().join("dest");
        
        fs::write(&source, "content")?;
        fs::create_dir(&dest_dir)?;

        Worker::execute(vec![(source.clone(), Action::Move(dest_dir.clone()))])?;

        assert!(!source.exists());
        assert!(dest_dir.join("source.txt").exists());
        Ok(())
    }
}
