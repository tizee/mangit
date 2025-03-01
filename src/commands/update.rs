use anyhow::{Result, Context};
use std::path::Path;
use std::process::Command;
use chrono::Utc;
use crate::storage::{self, find_repository, update_repository};

pub fn update(name_or_id: Option<String>) -> Result<()> {
    let storage = storage::default_storage()?;

    match name_or_id {
        Some(name) => update_single_repo(&storage, &name),
        None => update_all_repos(&storage),
    }
}

fn update_single_repo(storage: &impl storage::RepositoryStorage, name_or_id: &str) -> Result<()> {
    let mut repo = find_repository(storage, name_or_id)?;

    // Check if path exists
    let path = Path::new(&repo.path);
    if !path.exists() {
        println!("Warning: Repository path '{}' does not exist", repo.path);
        return Ok(());
    }

    println!("Updating repository '{}'...", repo.name);

    // Re-detect language
    repo.detect_language();

    // Update last modified timestamp
    repo.last_modified = Utc::now();

    // Optionally, run git fetch to check for remote updates
    let fetch_result = Command::new("git")
        .args(&["-C", &repo.path, "fetch"])
        .output();

    match fetch_result {
        Ok(_) => println!("Fetched latest changes for '{}'", repo.name),
        Err(_) => println!("Could not fetch changes for '{}'", repo.name),
    }

    // Update repository
    update_repository(storage, repo)
        .context("Failed to update repository")?;

    println!("Repository '{}' updated", name_or_id);

    Ok(())
}

fn update_all_repos(storage: &impl storage::RepositoryStorage) -> Result<()> {
    let mut repositories = storage.load()?;

    if repositories.is_empty() {
        println!("No repositories to update");
        return Ok(());
    }

    println!("Updating all repositories...");

    let mut updated_count = 0;

    for repo in &mut repositories {
        let path = Path::new(&repo.path);
        if !path.exists() {
            println!("Warning: Repository path '{}' does not exist for '{}'", repo.path, repo.name);
            continue;
        }

        println!("Updating '{}'...", repo.name);

        // Re-detect language
        repo.detect_language();

        // Update last modified timestamp
        repo.last_modified = Utc::now();

        // Optionally, run git fetch to check for remote updates
        let fetch_result = Command::new("git")
            .args(&["-C", &repo.path, "fetch"])
            .output();

        match fetch_result {
            Ok(_) => println!("Fetched latest changes for '{}'", repo.name),
            Err(_) => println!("Could not fetch changes for '{}'", repo.name),
        }

        updated_count += 1;
    }

    // Save repositories
    storage.save(&repositories)
        .context("Failed to save repositories")?;

    println!("Updated {} repositories", updated_count);

    Ok(())
}

#[cfg(test)]
mod tests_update {
    use super::*;
    use crate::repository::Repository;
    use chrono::Utc;
    use anyhow::anyhow;
    use std::sync::Mutex;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[test]
    fn test_update_single_repo() {
        // Arrange
        let repo_name = "test-repo";
        let temp_storage = TestStorage::new();

        // Add a test repository to the storage
        let repo = Repository {
            name: repo_name.to_string(),
            path: "/tmp/test-repo".to_string(), // Non-existent path to avoid git commands
            tags: vec!["test".to_string()],
            description: "Test repository".to_string(),
            last_modified: Utc::now(),
            language: None,
        };

        temp_storage.add_repository(repo);

        // Act
        let result = update_single_repo(&temp_storage, repo_name);

        // Assert - should be Ok even with non-existent path
        assert!(result.is_ok());

        // Check that the repository was "updated"
        let updated_repo = temp_storage.find_repository(repo_name).unwrap();
        assert_eq!(updated_repo.name, repo_name);
    }

    #[test]
    fn test_update_nonexistent_repo() {
        // Arrange
        let repo_name = "nonexistent-repo";
        let temp_storage = TestStorage::new();

        // Act
        let result = update_single_repo(&temp_storage, repo_name);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_update_all_repos() {
        // Arrange
        let temp_storage = TestStorage::new();

        // Add test repositories
        temp_storage.add_repository(Repository {
            name: "repo1".to_string(),
            path: "/tmp/repo1".to_string(), // Non-existent path to avoid git commands
            tags: vec!["test".to_string()],
            description: "Test repository 1".to_string(),
            last_modified: Utc::now(),
            language: None,
        });

        temp_storage.add_repository(Repository {
            name: "repo2".to_string(),
            path: "/tmp/repo2".to_string(), // Non-existent path to avoid git commands
            tags: vec!["test".to_string()],
            description: "Test repository 2".to_string(),
            last_modified: Utc::now(),
            language: None,
        });

        // Act
        let result = update_all_repos(&temp_storage);

        // Assert
        assert!(result.is_ok());

        // Check that the repositories were loaded and saved
        assert!(temp_storage.was_loaded());
        assert!(temp_storage.was_saved());
    }

    #[test]
    fn test_update_all_repos_empty() {
        // Arrange
        let temp_storage = TestStorage::new();

        // Act
        let result = update_all_repos(&temp_storage);

        // Assert
        assert!(result.is_ok());
        assert!(temp_storage.was_loaded());
        // No save should happen if there are no repositories
        assert!(!temp_storage.was_saved());
    }

    // Test storage implementation that tracks operations
    struct TestStorage {
        repositories: RefCell<HashMap<String, Repository>>,
        loaded: RefCell<bool>,
        saved: RefCell<bool>,
    }

    impl TestStorage {
        fn new() -> Self {
            Self {
                repositories: RefCell::new(HashMap::new()),
                loaded: RefCell::new(false),
                saved: RefCell::new(false),
            }
        }

        fn add_repository(&self, repo: Repository) {
            self.repositories.borrow_mut().insert(repo.name.clone(), repo);
        }

        fn find_repository(&self, name: &str) -> Option<Repository> {
            self.repositories.borrow().get(name).cloned()
        }

        fn was_loaded(&self) -> bool {
            *self.loaded.borrow()
        }

        fn was_saved(&self) -> bool {
            *self.saved.borrow()
        }
    }

    impl storage::RepositoryStorage for TestStorage {
        fn load(&self) -> Result<Vec<Repository>> {
            *self.loaded.borrow_mut() = true;
            Ok(self.repositories.borrow().values().cloned().collect())
        }

        fn save(&self, repositories: &[Repository]) -> Result<()> {
            *self.saved.borrow_mut() = true;

            // Update repositories
            let mut repo_map = self.repositories.borrow_mut();
            repo_map.clear();

            for repo in repositories {
                repo_map.insert(repo.name.clone(), repo.clone());
            }

            Ok(())
        }
    }
}
