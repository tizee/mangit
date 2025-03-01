use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepoAccess {
    pub tags: Vec<String>,
    pub access_times: Vec<DateTime<Utc>>,
}

impl RepoAccess {
    fn new(tags: Vec<String>) -> Self {
        RepoAccess {
            tags,
            access_times: vec![Utc::now()],
        }
    }

    fn record_access(&mut self) {
        self.access_times.push(Utc::now());
        // Keep only the last 10 access times to avoid unbounded growth
        if self.access_times.len() > 10 {
            self.access_times = self.access_times.split_off(self.access_times.len() - 10);
        }
    }

    fn update_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
        self.record_access();
    }

    fn reset_frequency(&mut self) {
        self.access_times = vec![Utc::now()];
    }

    fn calculate_frecency(&self) -> f64 {
        let now = Utc::now();
        let mut score = 0.0;

        for access_time in &self.access_times {
            let age = now.signed_duration_since(*access_time);

            // Weight based on recency
            let weight = if age < Duration::minutes(1) {
                100.0 // Within last minute
            } else if age < Duration::minutes(30) {
                80.0 // Within last 30 minutes
            } else if age < Duration::hours(1) {
                60.0 // Within last hour
            } else if age < Duration::hours(24) {
                40.0 // Within last day
            } else if age < Duration::hours(24 * 7) {
                20.0 // Within last week
            } else {
                10.0 // Older than a week
            };

            score += weight;
        }

        score
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Storage {
    // Map of absolute repo paths to their access information
    pub repos: HashMap<String, RepoAccess>,
}

impl Storage {
    /// Creates a new Storage instance, loading data from disk if available
    pub fn new(config: &Config) -> Result<Self> {
        config.ensure_mangit_dir()?;

        let repos_path = config.repos_path();
        if repos_path.exists() {
            let data = fs::read_to_string(&repos_path)
                .context("Failed to read repos file")?;
            let storage: Storage = serde_json::from_str(&data)
                .context("Failed to parse repos file")?;

            // Return a cleaned up storage (removing non-existent paths)
            let mut storage = storage;
            storage.cleanup();
            Ok(storage)
        } else {
            Ok(Storage::default())
        }
    }

    /// Saves the current storage state to disk
    pub fn save(&self, config: &Config) -> Result<()> {
        let repos_path = config.repos_path();
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize storage")?;
        fs::write(&repos_path, json)
            .context("Failed to write repos file")?;
        Ok(())
    }

    /// Converts a path to an absolute path
    fn to_absolute_path(path: &str) -> Result<String> {
        let path_buf = PathBuf::from(path);
        if path_buf.is_absolute() {
            Ok(path_buf.to_string_lossy().to_string())
        } else {
            let current_dir = env::current_dir()
                .context("Failed to get current directory")?;
            let abs_path = current_dir.join(path_buf);
            Ok(abs_path.to_string_lossy().to_string())
        }
    }

    /// Adds a repo with tags. Returns true if it's a new repo, false if updated
    pub fn add_repo(&mut self, path: &str, tags: Vec<String>) -> Result<bool> {
        let abs_path = Self::to_absolute_path(path)?;

        // Check if path exists
        if !Path::new(&abs_path).exists() {
            return Err(anyhow!("Path does not exist: {}", abs_path));
        }

        let is_new = !self.repos.contains_key(&abs_path);
        if is_new {
            self.repos.insert(abs_path, RepoAccess::new(tags));
        } else {
            if let Some(repo_access) = self.repos.get_mut(&abs_path) {
                repo_access.update_tags(tags);
            }
        }

        Ok(is_new)
    }

    /// Deletes a repo from storage. Returns true if found and deleted
    pub fn delete_repo(&mut self, path: &str) -> Result<bool> {
        let abs_path = Self::to_absolute_path(path)?;
        Ok(self.repos.remove(&abs_path).is_some())
    }

    /// Updates a repo's tags. Returns true if found and updated
    pub fn update_repo(&mut self, path: &str, tags: Vec<String>) -> Result<bool> {
        let abs_path = Self::to_absolute_path(path)?;

        if let Some(repo_access) = self.repos.get_mut(&abs_path) {
            repo_access.update_tags(tags);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Records an access to a repo. Returns true if found
    pub fn record_access(&mut self, path: &str) -> Result<bool> {
        let abs_path = Self::to_absolute_path(path)?;

        if let Some(repo_access) = self.repos.get_mut(&abs_path) {
            repo_access.record_access();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resets frequency for a specific repo or all repos if path is None
    pub fn reset_frequency(&mut self, path: Option<&str>) -> Result<usize> {
        match path {
            Some(path) => {
                let abs_path = Self::to_absolute_path(path)?;
                if let Some(repo_access) = self.repos.get_mut(&abs_path) {
                    repo_access.reset_frequency();
                    Ok(1)
                } else {
                    Ok(0)
                }
            },
            None => {
                // Reset frequency for all repos
                let count = self.repos.len();
                for (_, repo_access) in self.repos.iter_mut() {
                    repo_access.reset_frequency();
                }
                Ok(count)
            }
        }
    }

    /// Searches for repos by tag, returns paths sorted by frecency
    pub fn search_by_tag(&mut self, tag: &str) -> Vec<String> {
        let tag = tag.to_lowercase();

        // Collect matching repos and their frecency scores
        let mut matches: Vec<(String, f64)> = self.repos
            .iter_mut()
            .filter(|(_, repo_access)| {
                repo_access.tags.iter().any(|t| t.to_lowercase() == tag)
            })
            .map(|(path, repo_access)| {
                // Record access for each viewed repo
                repo_access.record_access();
                (path.clone(), repo_access.calculate_frecency())
            })
            .collect();

        // Sort by frecency score (descending)
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return just the paths
        matches.into_iter().map(|(path, _)| path).collect()
    }

    /// Removes repos with non-existent paths
    pub fn cleanup(&mut self) {
        self.repos.retain(|path, _| Path::new(path).exists());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use std::thread::sleep;
    use std::time::Duration as StdDuration;

    fn create_test_config() -> (Config, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let config = Config {
            mangit_dir: temp_dir.path().to_string_lossy().to_string(),
        };
        config.ensure_mangit_dir().unwrap();
        (config, temp_dir)
    }

    fn create_fake_repo(dir: &Path) -> PathBuf {
        let repo_path = dir.join("fake_repo");
        fs::create_dir_all(&repo_path).unwrap();
        fs::create_dir_all(repo_path.join(".git")).unwrap();
        repo_path
    }

    #[test]
    fn test_new_storage() {
        let (config, _temp_dir) = create_test_config();
        let storage = Storage::new(&config).unwrap();
        assert_eq!(storage.repos.len(), 0);
    }

    #[test]
    fn test_add_repo() {
        let (config, temp_dir) = create_test_config();
        let repo_path = create_fake_repo(temp_dir.path());

        let mut storage = Storage::new(&config).unwrap();
        let is_new = storage.add_repo(
            repo_path.to_str().unwrap(),
            vec!["test".to_string(), "rust".to_string()]
        ).unwrap();

        assert!(is_new);
        assert_eq!(storage.repos.len(), 1);

        // Test adding the same repo again
        let is_new = storage.add_repo(
            repo_path.to_str().unwrap(),
            vec!["updated".to_string()]
        ).unwrap();

        assert!(!is_new);
        assert_eq!(storage.repos.len(), 1);

        // Verify tags were updated
        let repo_access = storage.repos.get(repo_path.to_str().unwrap()).unwrap();
        assert_eq!(repo_access.tags, vec!["updated".to_string()]);
    }

    #[test]
    fn test_delete_repo() {
        let (config, temp_dir) = create_test_config();
        let repo_path = create_fake_repo(temp_dir.path());

        let mut storage = Storage::new(&config).unwrap();
        storage.add_repo(
            repo_path.to_str().unwrap(),
            vec!["test".to_string()]
        ).unwrap();

        assert_eq!(storage.repos.len(), 1);

        let deleted = storage.delete_repo(repo_path.to_str().unwrap()).unwrap();
        assert!(deleted);
        assert_eq!(storage.repos.len(), 0);

        // Test deleting non-existent repo
        let deleted = storage.delete_repo("non-existent-path").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_update_repo() {
        let (config, temp_dir) = create_test_config();
        let repo_path = create_fake_repo(temp_dir.path());

        let mut storage = Storage::new(&config).unwrap();
        storage.add_repo(
            repo_path.to_str().unwrap(),
            vec!["initial".to_string()]
        ).unwrap();

        let updated = storage.update_repo(
            repo_path.to_str().unwrap(),
            vec!["updated".to_string(), "tags".to_string()]
        ).unwrap();

        assert!(updated);

        let repo_access = storage.repos.get(repo_path.to_str().unwrap()).unwrap();
        assert_eq!(repo_access.tags, vec!["updated".to_string(), "tags".to_string()]);

        // Test updating non-existent repo
        let updated = storage.update_repo("non-existent-path", vec!["tag".to_string()]).unwrap();
        assert!(!updated);
    }

    #[test]
    fn test_reset_frequency() {
        let (config, temp_dir) = create_test_config();
        let repo_path = create_fake_repo(temp_dir.path());

        let mut storage = Storage::new(&config).unwrap();
        storage.add_repo(
            repo_path.to_str().unwrap(),
            vec!["test".to_string()]
        ).unwrap();

        // Record some accesses
        for _ in 0..3 {
            storage.record_access(repo_path.to_str().unwrap()).unwrap();
            sleep(StdDuration::from_millis(10));
        }

        // Check that there are multiple access times
        let repo_access = storage.repos.get(repo_path.to_str().unwrap()).unwrap();
        assert!(repo_access.access_times.len() > 1);

        // Reset frequency for the specific repo
        let reset_count = storage.reset_frequency(Some(repo_path.to_str().unwrap())).unwrap();
        assert_eq!(reset_count, 1);

        // Verify that access times were reset
        let repo_access = storage.repos.get(repo_path.to_str().unwrap()).unwrap();
        assert_eq!(repo_access.access_times.len(), 1);

        // Test resetting all repos
        let repo2 = create_fake_repo(&temp_dir.path().join("repo2"));
        storage.add_repo(
            repo2.to_str().unwrap(),
            vec!["test".to_string()]
        ).unwrap();

        // Record more accesses
        for _ in 0..2 {
            storage.record_access(repo_path.to_str().unwrap()).unwrap();
            storage.record_access(repo2.to_str().unwrap()).unwrap();
            sleep(StdDuration::from_millis(10));
        }

        // Reset all
        let reset_count = storage.reset_frequency(None).unwrap();
        assert_eq!(reset_count, 2);

        // Verify all were reset
        for (_, repo_access) in storage.repos.iter() {
            assert_eq!(repo_access.access_times.len(), 1);
        }
    }

    #[test]
    fn test_search_by_tag() {
        let (config, temp_dir) = create_test_config();
        let repo1 = create_fake_repo(&temp_dir.path().join("repo1"));
        let repo2 = create_fake_repo(&temp_dir.path().join("repo2"));
        let repo3 = create_fake_repo(&temp_dir.path().join("repo3"));

        let mut storage = Storage::new(&config).unwrap();

        // Add repos with different tags
        storage.add_repo(
            repo1.to_str().unwrap(),
            vec!["rust".to_string(), "cli".to_string()]
        ).unwrap();

        storage.add_repo(
            repo2.to_str().unwrap(),
            vec!["rust".to_string(), "web".to_string()]
        ).unwrap();

        storage.add_repo(
            repo3.to_str().unwrap(),
            vec!["python".to_string(), "cli".to_string()]
        ).unwrap();

        // Test searching by tag
        let rust_repos = storage.search_by_tag("rust");
        assert_eq!(rust_repos.len(), 2);
        assert!(rust_repos.contains(&repo1.to_str().unwrap().to_string()));
        assert!(rust_repos.contains(&repo2.to_str().unwrap().to_string()));

        let cli_repos = storage.search_by_tag("cli");
        assert_eq!(cli_repos.len(), 2);
        assert!(cli_repos.contains(&repo1.to_str().unwrap().to_string()));
        assert!(cli_repos.contains(&repo3.to_str().unwrap().to_string()));

        let web_repos = storage.search_by_tag("web");
        assert_eq!(web_repos.len(), 1);
        assert!(web_repos.contains(&repo2.to_str().unwrap().to_string()));

        // Test searching by non-existent tag
        let empty_repos = storage.search_by_tag("nonexistent");
        assert_eq!(empty_repos.len(), 0);
    }

    #[test]
    fn test_frecency_sorting() {
        let (config, temp_dir) = create_test_config();
        let repo1 = create_fake_repo(&temp_dir.path().join("repo1"));
        let repo2 = create_fake_repo(&temp_dir.path().join("repo2"));

        let mut storage = Storage::new(&config).unwrap();

        // Add repos with the same tag
        storage.add_repo(
            repo1.to_str().unwrap(),
            vec!["common".to_string()]
        ).unwrap();

        storage.add_repo(
            repo2.to_str().unwrap(),
            vec!["common".to_string()]
        ).unwrap();

        // Access repo2 more frequently
        for _ in 0..3 {
            storage.record_access(repo2.to_str().unwrap()).unwrap();
            sleep(StdDuration::from_millis(10)); // Small delay to ensure different timestamps
        }

        // Search by common tag, repo2 should come first due to higher frecency
        let results = storage.search_by_tag("common");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], repo2.to_str().unwrap().to_string());
        assert_eq!(results[1], repo1.to_str().unwrap().to_string());
    }

    #[test]
    fn test_cleanup() {
        let (config, temp_dir) = create_test_config();
        let repo_path = create_fake_repo(temp_dir.path());

        let mut storage = Storage::new(&config).unwrap();
        storage.add_repo(
            repo_path.to_str().unwrap(),
            vec!["test".to_string()]
        ).unwrap();

        // Add a non-existent path directly to the HashMap
        let non_existent = "/path/does/not/exist";
        storage.repos.insert(
            non_existent.to_string(),
            RepoAccess::new(vec!["fake".to_string()])
        );

        assert_eq!(storage.repos.len(), 2);

        // Run cleanup
        storage.cleanup();

        // Only the real repo should remain
        assert_eq!(storage.repos.len(), 1);
        assert!(storage.repos.contains_key(repo_path.to_str().unwrap()));
        assert!(!storage.repos.contains_key(non_existent));
    }

    #[test]
    fn test_save_and_load() {
        let (config, temp_dir) = create_test_config();
        let repo_path = create_fake_repo(temp_dir.path());

        // Create and save storage
        let mut storage = Storage::new(&config).unwrap();
        storage.add_repo(
            repo_path.to_str().unwrap(),
            vec!["test".to_string(), "save".to_string()]
        ).unwrap();

        storage.save(&config).unwrap();

        // Load storage from the saved file
        let loaded_storage = Storage::new(&config).unwrap();

        assert_eq!(loaded_storage.repos.len(), 1);
        assert!(loaded_storage.repos.contains_key(repo_path.to_str().unwrap()));

        let loaded_tags = &loaded_storage.repos.get(repo_path.to_str().unwrap()).unwrap().tags;
        assert_eq!(loaded_tags.len(), 2);
        assert!(loaded_tags.contains(&"test".to_string()));
        assert!(loaded_tags.contains(&"save".to_string()));
    }
}
