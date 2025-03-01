// remove.rs
use anyhow::Result;
use crate::storage::{default_storage, remove_repository};
use crate::storage::RepositoryStorage;

/// 使用传入的 storage 删除指定名称或 id 的仓库，用于依赖注入和测试。
pub fn remove_with_storage(storage: &impl RepositoryStorage, name_or_id: &str) -> Result<()> {
    remove_repository(storage, name_or_id)?;
    println!("Removed repository '{}'", name_or_id);
    Ok(())
}

/// 使用默认 storage 删除指定名称或 id 的仓库。
pub fn remove(name_or_id: &str) -> Result<()> {
    let storage = default_storage()?;
    remove_with_storage(&storage, name_or_id)
}

#[cfg(test)]
mod tests_remove {
    use super::*;
    use crate::repository::Repository;
    use anyhow::Result;
    use chrono::Utc;
    use std::cell::RefCell;

    /// 测试用的 storage 实现，通过内部 RefCell 保存仓库列表，方便测试时检查变更。
    struct TestStorage {
        repositories: RefCell<Vec<Repository>>,
    }

    impl TestStorage {
        fn new(repositories: Vec<Repository>) -> Self {
            TestStorage {
                repositories: RefCell::new(repositories),
            }
        }
    }

    impl crate::storage::RepositoryStorage for TestStorage {
        fn load(&self) -> Result<Vec<Repository>> {
            Ok(self.repositories.borrow().clone())
        }

        fn save(&self, repositories: &[Repository]) -> Result<()> {
            *self.repositories.borrow_mut() = repositories.to_vec();
            Ok(())
        }
    }

    #[test]
    fn test_remove_existing_repository() {
        let test_repo_name = "test-repo";
        // 构造存在的测试仓库
        let test_repo = Repository {
            name: test_repo_name.to_string(),
            path: "/path/to/repo".to_string(),
            tags: vec!["test".to_string()],
            description: "Test repository".to_string(),
            last_modified: Utc::now(),
            language: None,
        };

        let storage = TestStorage::new(vec![test_repo]);
        // 执行删除操作
        let result = remove_with_storage(&storage, test_repo_name);
        // 断言删除成功，且仓库列表为空
        assert!(result.is_ok());
        let repos = storage.load().unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_repository() {
        let non_existent_repo = "non-existent-repo";
        // 构造不包含目标仓库的测试数据
        let test_repo = Repository {
            name: "test-repo".to_string(),
            path: "/path/to/repo".to_string(),
            tags: vec!["test".to_string()],
            description: "Test repository".to_string(),
            last_modified: Utc::now(),
            language: None,
        };

        let storage = TestStorage::new(vec![test_repo]);
        // 执行删除不存在的仓库操作
        let result = remove_with_storage(&storage, non_existent_repo);
        // 断言删除失败，错误信息包含 "not found"，且仓库列表未改变
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("not found"));
        let repos = storage.load().unwrap();
        assert_eq!(repos.len(), 1);
    }
}
