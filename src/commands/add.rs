// add.rs
use anyhow::{Result, anyhow, Context};
use std::path::Path;
use crate::config::is_git_repo;
use crate::repository::Repository;
use crate::storage::{self, add_repository};
use crate::storage::RepositoryStorage;

/// 使用传入的 storage 添加仓库，便于依赖注入和测试
pub fn add_with_storage(
    storage: &impl RepositoryStorage,
    path: &str,
    name: Option<String>,
    tags: Option<String>,
    desc: Option<String>,
) -> Result<()> {
    // 展开路径中的 tilde
    let expanded_path = shellexpand::tilde(path);
    let expanded_str = expanded_path.as_ref();
    let path_obj = Path::new(expanded_str);

    // 检查路径是否存在
    if !path_obj.exists() {
        return Err(anyhow!("Path '{}' does not exist", path));
    }

    // 检查路径是否为 Git 仓库
    if !is_git_repo(path)? {
        return Err(anyhow!("Path '{}' is not a Git repository", path));
    }

    // 确定仓库名称：若未提供则使用目录名
    let repo_name = if let Some(n) = name {
        n
    } else {
        match path_obj.file_name() {
            Some(os_str) => os_str.to_string_lossy().to_string(),
            None => return Err(anyhow!("Could not determine repository name from path")),
        }
    };

    // 解析 tags：将逗号分隔的字符串分割后，去除空白部分
    let repo_tags = if let Some(t) = tags {
        let mut vec = Vec::new();
        for s in t.split(',') {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                vec.push(trimmed.to_string());
            }
        }
        vec
    } else {
        Vec::new()
    };

    // 创建 repository 对象
    let mut repository = Repository::new(
        repo_name.clone(),
        path_obj.to_string_lossy().to_string(),
        repo_tags,
        desc.unwrap_or_default(),
    );

    // 自动检测语言
    repository.detect_language();

    // 添加仓库到存储中
    add_repository(storage, repository)
        .context("Failed to add repository")?;

    println!("Added repository '{}' at '{}'", repo_name, path);

    Ok(())
}

/// 使用默认 storage 添加仓库
pub fn add(
    path: &str,
    name: Option<String>,
    tags: Option<String>,
    desc: Option<String>,
) -> Result<()> {
    let storage = storage::default_storage()?;
    add_with_storage(&storage, path, name, tags, desc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;
    use anyhow::Result;
    use crate::repository::Repository;
    use crate::storage::RepositoryStorage;
    use std::cell::RefCell;

    // 测试用的存储实现，使用 RefCell 模拟内存存储
    struct TestStorage {
        repositories: RefCell<Vec<Repository>>,
    }

    impl TestStorage {
        fn new() -> Self {
            TestStorage {
                repositories: RefCell::new(Vec::new()),
            }
        }
    }

    impl RepositoryStorage for TestStorage {
        fn load(&self) -> Result<Vec<Repository>> {
            Ok(self.repositories.borrow().clone())
        }

        fn save(&self, repositories: &[Repository]) -> Result<()> {
            *self.repositories.borrow_mut() = repositories.to_vec();
            Ok(())
        }
    }

    // 辅助函数：在指定目录中创建 .git 子目录模拟 Git 仓库
    fn create_git_repo(dir: &Path) -> Result<()> {
        fs::create_dir_all(dir.join(".git"))?;
        Ok(())
    }

    #[test]
    fn test_add_with_existing_git_repo() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path).unwrap();

        let test_storage = TestStorage::new();

        // Act
        let result = add_with_storage(
            &test_storage,
            repo_path.to_str().unwrap(),
            Some("test-repo".to_string()),
            Some("rust,cli".to_string()),
            Some("Test repository".to_string()),
        );

        // Assert
        assert!(result.is_ok());
        let repos = test_storage.load().unwrap();
        assert_eq!(repos.len(), 1);
        let repo = &repos[0];
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.tags, vec!["rust", "cli"]);
        assert_eq!(repo.description, "Test repository");
    }

    #[test]
    fn test_add_with_non_existent_path() {
        // Arrange
        let non_existent_path = "/path/does/not/exist";
        let test_storage = TestStorage::new();

        // Act
        let result = add_with_storage(
            &test_storage,
            non_existent_path,
            Some("test-repo".to_string()),
            None,
            None,
        );

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not exist"));
    }

    #[test]
    fn test_add_with_non_git_repo() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path(); // 未创建 .git 子目录
        let test_storage = TestStorage::new();

        // Act
        let result = add_with_storage(
            &test_storage,
            repo_path.to_str().unwrap(),
            Some("test-repo".to_string()),
            None,
            None,
        );

        // Assert
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not a Git repository"));
    }

    #[test]
    fn test_add_with_no_name_provided() {
        // Arrange
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path();
        create_git_repo(repo_path).unwrap();
        let dir_name = match repo_path.file_name() {
            Some(os_str) => os_str.to_string_lossy().to_string(),
            None => String::new(),
        };

        let test_storage = TestStorage::new();

        // Act
        let result = add_with_storage(
            &test_storage,
            repo_path.to_str().unwrap(),
            None, // 未提供名称
            Some("rust,cli".to_string()),
            Some("Test repository".to_string()),
        );

        // Assert
        assert!(result.is_ok());
        let repos = test_storage.load().unwrap();
        assert_eq!(repos.len(), 1);
        let repo = &repos[0];
        assert_eq!(repo.name, dir_name);
    }
}
