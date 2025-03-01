use anyhow::Result;
use std::process::Command;
use chrono::Utc;
use crate::storage;
use crate::repository::Repository;

/// RepoInfoPrinter 封装了仓库存储对象以及查找仓库的逻辑，避免全局函数重写，便于依赖注入。
pub struct RepoInfoPrinter<S> {
    storage: S,
    find_repository: fn(&S, &str) -> Result<Repository>,
}

impl<S> RepoInfoPrinter<S> {
    /// 创建新的 RepoInfoPrinter 实例，传入存储对象和查找仓库的命名函数。
    pub fn new(storage: S, find_repository: fn(&S, &str) -> Result<Repository>) -> Self {
        Self { storage, find_repository }
    }

    /// 打印仓库信息，包括基本信息和 git 状态、最近提交记录。
    pub fn print_info(&self, name_or_id: &str) -> Result<()> {
        // 根据传入的查找函数获取仓库信息
        let repo = (self.find_repository)(&self.storage, name_or_id)?;

        // 计算上次修改距离当前天数
        let days_ago = (Utc::now() - repo.last_modified).num_days();
        let modified_str = if days_ago == 0 {
            "Today".to_string()
        } else if days_ago == 1 {
            "Yesterday".to_string()
        } else {
            format!("{} days ago", days_ago)
        };

        let tags_str = if repo.tags.is_empty() {
            "None".to_string()
        } else {
            repo.tags.join(", ")
        };

        // 打印仓库基本信息
        println!("Repository: {}", repo.name);
        println!("Path: {}", repo.path);
        println!("Language: {}", repo.language.unwrap_or("Unknown".to_string()));
        println!("Tags: {}", tags_str);
        println!("Last Modified: {}", modified_str);
        println!("Description: {}", if repo.description.is_empty() { "None" } else { &repo.description });

        // 执行 git 命令，获取状态和最近提交记录
        let path = &repo.path;

        println!("\nGit Status:");
        let status_result = Command::new("git")
            .args(&["-C", path, "status", "--short"])
            .output();

        match status_result {
            Ok(output) => {
                let status_str = String::from_utf8_lossy(&output.stdout);
                if status_str.trim().is_empty() {
                    println!("  No changes");
                } else {
                    for line in status_str.lines() {
                        println!("  {}", line);
                    }
                }
            },
            Err(_) => {
                println!("  Could not get git status");
            }
        }

        println!("\nRecent Commits:");
        let log_result = Command::new("git")
            .args(&["-C", path, "log", "--pretty=oneline", "-n", "5"])
            .output();

        match log_result {
            Ok(output) => {
                let log_str = String::from_utf8_lossy(&output.stdout);
                if log_str.trim().is_empty() {
                    println!("  No commits");
                } else {
                    for line in log_str.lines() {
                        println!("  {}", line);
                    }
                }
            },
            Err(_) => {
                println!("  Could not get git log");
            }
        }

        Ok(())
    }
}

/// 生产环境 API，调用默认存储和查找函数
pub fn info(name_or_id: &str) -> Result<()> {
    let storage_instance = storage::default_storage()?;
    let printer = RepoInfoPrinter::new(storage_instance, storage::find_repository);
    printer.print_info(name_or_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    use chrono::Utc;
    use anyhow::anyhow;

    // 测试中使用的简单存储实现
    struct TestStorage;

    impl storage::RepositoryStorage for TestStorage {
        fn load(&self) -> Result<Vec<Repository>> {
            Ok(Vec::new())
        }

        fn save(&self, _repositories: &[Repository]) -> Result<()> {
            Ok(())
        }
    }

    /// 命名函数：测试成功时根据仓库名称返回仓库信息
    fn test_find_repository_success(_storage: &TestStorage, name: &str) -> Result<Repository> {
        if name == "test-repo" {
            Ok(Repository {
                name: name.to_string(),
                path: "/tmp/nonexistent/path".to_string(), // 避免真实执行 git 命令
                tags: vec!["test".to_string(), "rust".to_string()],
                description: "Test repository".to_string(),
                last_modified: Utc::now(),
                language: Some("Rust".to_string()),
            })
        } else {
            Err(anyhow!("Repository not found"))
        }
    }

    /// 命名函数：测试失败时返回错误
    fn test_find_repository_failure(_storage: &TestStorage, name: &str) -> Result<Repository> {
        Err(anyhow!("Repository '{}' not found", name))
    }

    #[test]
    fn test_info_for_repository() {
        let printer = RepoInfoPrinter::new(TestStorage, test_find_repository_success);
        let result = printer.print_info("test-repo");
        assert!(result.is_ok());
    }

    #[test]
    fn test_info_nonexistent_repository() {
        let printer = RepoInfoPrinter::new(TestStorage, test_find_repository_failure);
        let result = printer.print_info("nonexistent-repo");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not found"));
    }
}
