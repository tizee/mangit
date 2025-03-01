use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub mangit_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("~"));
        Config {
            mangit_dir: home.join(".mangit").to_string_lossy().to_string(),
        }
    }
}

impl Config {
    /// Returns the mangit directory as PathBuf
    pub fn mangit_dir_path(&self) -> PathBuf {
        PathBuf::from(&self.mangit_dir)
    }

    /// Returns the repos file path
    pub fn repos_path(&self) -> PathBuf {
        self.mangit_dir_path().join("repos.json")
    }

    /// Ensures the mangit directory exists
    pub fn ensure_mangit_dir(&self) -> Result<()> {
        let dir = self.mangit_dir_path();
        if !dir.exists() {
            fs::create_dir_all(&dir).context("Failed to create mangit directory")?;
        }
        Ok(())
    }
}

/// Checks if a path is a valid git repository
pub fn is_git_repo(path: &Path) -> bool {
    if !path.exists() {
        return false;
    }
    path.join(".git").exists() && path.join(".git").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.mangit_dir.is_empty());
        assert!(config.mangit_dir.contains(".mangit"));
    }

    #[test]
    fn test_mangit_dir_path() {
        let temp_dir = tempdir().unwrap();
        let config = Config {
            mangit_dir: temp_dir
                .path()
                .join(".mangit")
                .to_string_lossy()
                .to_string(),
        };
        let expected_path = temp_dir.path().join(".mangit");
        assert_eq!(config.mangit_dir_path(), expected_path);
    }

    #[test]
    fn test_ensure_mangit_dir_creates_dir() {
        let temp_dir = tempdir().unwrap();
        let expected_dir = temp_dir.path().join(".mangit");
        let config = Config {
            mangit_dir: expected_dir.to_string_lossy().to_string(),
        };
        assert!(!expected_dir.exists());
        let result = config.ensure_mangit_dir();
        assert!(result.is_ok());
        assert!(expected_dir.exists());
    }

    #[test]
    fn test_is_git_repo() {
        let temp_dir = tempdir().unwrap();
        assert!(!is_git_repo(temp_dir.path()));

        // Create .git directory to make it a git repo
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        assert!(is_git_repo(temp_dir.path()));
    }
}
