use anyhow::{Result, Context, anyhow};
use std::fs;
use std::path::PathBuf;
use crate::config::Config;
use crate::repository::Repository;
use serde_json;

/// 存储仓库数据的 Trait
pub trait RepositoryStorage {
    fn load(&self) -> Result<Vec<Repository>>;
    fn save(&self, repositories: &[Repository]) -> Result<()>;
}

/// 基于文件存储的实现，内部保存 repos_path 为 PathBuf
pub struct FileStorage {
    repos_path: PathBuf,
}

impl FileStorage {
    /// 直接传入 repos_path 构造 FileStorage 实例
    pub fn new(repos_path: PathBuf) -> Self {
        FileStorage { repos_path }
    }

    /// 根据传入的 Config 构造 FileStorage
    pub fn from_config(config: &Config) -> Result<Self> {
        Ok(Self::new(config.repos_path()))
    }
}

impl RepositoryStorage for FileStorage {
    fn load(&self) -> Result<Vec<Repository>> {
        if !self.repos_path.exists() {
            return Ok(Vec::new());
        }
        let repos_str = fs::read_to_string(&self.repos_path)
            .context("Failed to read repositories file")?;
        let repositories: Vec<Repository> = serde_json::from_str(&repos_str)
            .context("Failed to parse repositories file")?;
        Ok(repositories)
    }

    fn save(&self, repositories: &[Repository]) -> Result<()> {
        let parent_dir = match self.repos_path.parent() {
            Some(p) => p,
            None => return Err(anyhow!("Invalid repository path")),
        };
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .context("Failed to create parent directory")?;
        }
        let repos_str = serde_json::to_string_pretty(repositories)
            .context("Failed to serialize repositories")?;
        fs::write(&self.repos_path, repos_str)
            .context("Failed to write repositories file")
    }
}

/// 使用默认 Config 创建默认的文件存储
pub fn default_storage() -> Result<FileStorage> {
    let config = Config::default();
    config.ensure_mangit_dir()?;
    Ok(FileStorage::new(config.repos_path()))
}

/// 添加仓库：若同名仓库已存在，则返回错误
pub fn add_repository(storage: &impl RepositoryStorage, repository: Repository) -> Result<()> {
    let repositories = storage.load()?;
    let mut duplicate_found = false;
    let mut repos_new = Vec::new();
    for repo in repositories {
        if repo.name == repository.name {
            duplicate_found = true;
            break;
        }
        repos_new.push(repo);
    }
    if duplicate_found {
        return Err(anyhow!(format!(
            "Repository with name '{}' already exists",
            repository.name
        )));
    }
    repos_new.push(repository);
    storage.save(&repos_new)
}

/// 删除指定仓库，若未找到返回错误
pub fn remove_repository(storage: &impl RepositoryStorage, name_or_id: &str) -> Result<()> {
    let repositories = storage.load()?;
    let mut new_repositories = Vec::new();
    let mut found = false;
    for repo in repositories {
        if repo.name == name_or_id {
            found = true;
        } else {
            new_repositories.push(repo);
        }
    }
    if !found {
        return Err(anyhow!(format!(
            "Repository '{}' not found",
            name_or_id
        )));
    }
    storage.save(&new_repositories)
}

/// 查找指定仓库（通过名称），未找到则返回错误
pub fn find_repository(storage: &impl RepositoryStorage, name_or_id: &str) -> Result<Repository> {
    let repositories = storage.load()?;
    for repo in &repositories {
        if repo.name == name_or_id {
            return Ok(repo.clone());
        }
    }
    Err(anyhow!(format!(
        "Repository '{}' not found",
        name_or_id
    )))
}

/// 更新指定仓库（通过名称匹配），未找到则返回错误
pub fn update_repository(storage: &impl RepositoryStorage, repository: Repository) -> Result<()> {
    let mut repositories = storage.load()?;
    let mut index_opt: Option<usize> = None;
    for (index, repo) in repositories.iter().enumerate() {
        if repo.name == repository.name {
            index_opt = Some(index);
            break;
        }
    }
    match index_opt {
        Some(index) => {
            repositories[index] = repository;
            storage.save(&repositories)
        },
        None => Err(anyhow!(format!(
            "Repository '{}' not found",
            repository.name
        ))),
    }
}

/// 兼容已有代码：加载仓库列表
pub fn load_repositories() -> Result<Vec<Repository>> {
    let storage = default_storage()?;
    storage.load()
}

/// 兼容已有代码：保存仓库列表
pub fn save_repositories(repositories: &[Repository]) -> Result<()> {
    let storage = default_storage()?;
    storage.save(repositories)
}

#[cfg(test)]
mod tests_storage {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;
    use chrono::Utc;
    use crate::repository::Repository;

    /// 测试用存储实现，使用临时目录存储数据
    struct TestStorage {
        path: PathBuf,
    }

    impl TestStorage {
        fn new() -> Self {
            let dir = tempdir().unwrap();
            let path = dir.path().join("repos.json");
            // 为测试保留临时目录生命周期，短暂泄露内存无妨
            std::mem::forget(dir);
            Self { path }
        }
    }

    impl RepositoryStorage for TestStorage {
        fn load(&self) -> Result<Vec<Repository>> {
            if !self.path.exists() {
                return Ok(Vec::new());
            }
            let repos_str = fs::read_to_string(&self.path)
                .context("Failed to read test repositories file")?;
            let repositories: Vec<Repository> = serde_json::from_str(&repos_str)
                .context("Failed to parse test repositories file")?;
            Ok(repositories)
        }

        fn save(&self, repositories: &[Repository]) -> Result<()> {
            let parent = match self.path.parent() {
                Some(p) => p,
                None => return Err(anyhow!("Invalid test repository path")),
            };
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .context("Failed to create test parent directory")?;
            }
            let repos_str = serde_json::to_string_pretty(repositories)
                .context("Failed to serialize test repositories")?;
            fs::write(&self.path, repos_str)
                .context("Failed to write test repositories file")
        }
    }

    /// 辅助函数：构造测试仓库数据
    fn create_test_repositories() -> Vec<Repository> {
        vec![
            Repository {
                name: "repo1".to_string(),
                path: "/path/to/repo1".to_string(),
                tags: vec!["rust".to_string(), "cli".to_string()],
                description: "Test repository 1".to_string(),
                last_modified: Utc::now(),
                language: Some("Rust".to_string()),
            },
            Repository {
                name: "repo2".to_string(),
                path: "/path/to/repo2".to_string(),
                tags: vec!["javascript".to_string(), "web".to_string()],
                description: "Test repository 2".to_string(),
                last_modified: Utc::now(),
                language: Some("JavaScript/TypeScript".to_string()),
            },
        ]
    }

    #[test]
    fn test_load_repositories_empty() {
        let storage = TestStorage::new();
        let repos = storage.load();
        assert!(repos.is_ok());
        assert!(repos.unwrap().is_empty());
    }

    #[test]
    fn test_load_repositories_with_content() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        storage.save(&test_repos).unwrap();
        let repos = storage.load();
        assert!(repos.is_ok());
        let loaded_repos = repos.unwrap();
        assert_eq!(loaded_repos.len(), 2);
        assert_eq!(loaded_repos[0].name, "repo1");
        assert_eq!(loaded_repos[1].name, "repo2");
    }

    #[test]
    fn test_save_repositories() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        let result = storage.save(&test_repos);
        assert!(result.is_ok());
        let loaded_repos = storage.load().unwrap();
        assert_eq!(loaded_repos.len(), 2);
        assert_eq!(loaded_repos[0].name, "repo1");
        assert_eq!(loaded_repos[1].name, "repo2");
    }

    #[test]
    fn test_add_repository() {
        let storage = TestStorage::new();
        let mut initial_repos = Vec::new();
        initial_repos.push(create_test_repositories()[0].clone());
        storage.save(&initial_repos).unwrap();
        let new_repo = Repository {
            name: "new-repo".to_string(),
            path: "/path/to/new-repo".to_string(),
            tags: vec!["python".to_string(), "data-science".to_string()],
            description: "A new test repository".to_string(),
            last_modified: Utc::now(),
            language: Some("Python".to_string()),
        };
        let result = add_repository(&storage, new_repo.clone());
        assert!(result.is_ok());
        let loaded_repos = storage.load().unwrap();
        assert_eq!(loaded_repos.len(), 2);
        let mut found = false;
        for repo in loaded_repos {
            if repo.name == "new-repo" {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_add_repository_duplicate() {
        let storage = TestStorage::new();
        let initial_repos = create_test_repositories();
        storage.save(&initial_repos).unwrap();
        let duplicate_repo = Repository {
            name: "repo1".to_string(),
            path: "/path/to/other-repo".to_string(),
            tags: vec!["duplicate".to_string()],
            description: "A duplicate repo".to_string(),
            last_modified: Utc::now(),
            language: None,
        };
        let result = add_repository(&storage, duplicate_repo);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("already exists"));
    }

    #[test]
    fn test_remove_repository() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        storage.save(&test_repos).unwrap();
        let result = remove_repository(&storage, "repo1");
        assert!(result.is_ok());
        let loaded_repos = storage.load().unwrap();
        assert_eq!(loaded_repos.len(), 1);
        assert_eq!(loaded_repos[0].name, "repo2");
    }

    #[test]
    fn test_remove_repository_not_found() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        storage.save(&test_repos).unwrap();
        let result = remove_repository(&storage, "non-existent-repo");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_find_repository() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        storage.save(&test_repos).unwrap();
        let result = find_repository(&storage, "repo2");
        assert!(result.is_ok());
        let repo = result.unwrap();
        assert_eq!(repo.name, "repo2");
        assert_eq!(repo.path, "/path/to/repo2");
    }

    #[test]
    fn test_find_repository_not_found() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        storage.save(&test_repos).unwrap();
        let result = find_repository(&storage, "non-existent-repo");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }

    #[test]
    fn test_update_repository() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        storage.save(&test_repos).unwrap();
        let mut updated_repo = test_repos[0].clone();
        updated_repo.description = "Updated description".to_string();
        updated_repo.tags = vec!["rust".to_string(), "cli".to_string(), "updated".to_string()];
        let result = update_repository(&storage, updated_repo.clone());
        assert!(result.is_ok());
        let loaded_repos = storage.load().unwrap();
        let mut found = false;
        for repo in loaded_repos {
            if repo.name == "repo1" {
                found = true;
                assert_eq!(repo.description, "Updated description");
                assert!(repo.tags.len() == 3);
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_update_repository_not_found() {
        let storage = TestStorage::new();
        let test_repos = create_test_repositories();
        storage.save(&test_repos).unwrap();
        let non_existent_repo = Repository {
            name: "non-existent-repo".to_string(),
            path: "/path/to/nowhere".to_string(),
            tags: vec![],
            description: "This repo doesn't exist".to_string(),
            last_modified: Utc::now(),
            language: None,
        };
        let result = update_repository(&storage, non_existent_repo);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
    }
}
