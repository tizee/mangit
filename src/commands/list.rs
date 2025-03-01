// list.rs
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::cmp::Ordering;
use crate::repository::Repository;
use crate::storage;
use crate::storage::RepositoryStorage;

/// 过滤并排序仓库，纯函数，无副作用
pub fn get_filtered_repositories(
    repositories: &[Repository],
    tags: &Option<String>,
    sort: &Option<String>,
) -> Vec<Repository> {
    // 复制所有仓库数据
    let mut filtered_repos = repositories.to_vec();

    // 若提供了 tags，则进行过滤
    if let Some(tags_str) = tags {
        let mut tag_list: Vec<String> = Vec::new();
        for part in tags_str.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                tag_list.push(trimmed.to_string());
            }
        }
        let mut filtered: Vec<Repository> = Vec::new();
        let total = filtered_repos.len();
        let mut i = 0;
        while i < total {
            if filtered_repos[i].matches_tags(&tag_list) {
                filtered.push(filtered_repos[i].clone());
            }
            i += 1;
        }
        filtered_repos = filtered;
    }

    // 排序
    filtered_repos = sort_repositories(filtered_repos, sort);

    filtered_repos
}

/// 根据名称进行比较（不区分大小写）
fn compare_by_name(a: &Repository, b: &Repository) -> Ordering {
    a.name.to_lowercase().cmp(&b.name.to_lowercase())
}

/// 根据最后修改时间进行比较（最近的排在前面）
fn compare_by_modified(a: &Repository, b: &Repository) -> Ordering {
    b.last_modified.cmp(&a.last_modified)
}

/// 根据指定的排序选项对仓库进行排序
fn sort_repositories(mut repos: Vec<Repository>, sort: &Option<String>) -> Vec<Repository> {
    if let Some(sort_field) = sort {
        if sort_field == "name" {
            repos.sort_by(compare_by_name);
        } else if sort_field == "modified" {
            repos.sort_by(compare_by_modified);
        } else {
            println!("Unknown sort field '{}', using default", sort_field);
            repos.sort_by(compare_by_name);
        }
    } else {
        repos.sort_by(compare_by_name);
    }
    repos
}

/// 根据时间戳格式化相对时间
fn format_time_ago(timestamp: DateTime<Utc>) -> String {
    let days_ago = (Utc::now() - timestamp).num_days();
    if days_ago == 0 {
        "Today".to_string()
    } else if days_ago == 1 {
        "Yesterday".to_string()
    } else {
        format!("{} days ago", days_ago)
    }
}

/// 以表格形式显示仓库信息
fn display_repositories(repositories: &[Repository]) {
    if repositories.is_empty() {
        println!("No repositories found");
        return;
    }

    // 打印表头
    println!(
        "{:<20} {:<15} {:<40} {:<20}",
        "NAME", "LANGUAGE", "TAGS", "LAST MODIFIED"
    );
    println!("{}", "-".repeat(95));

    // 打印每条仓库记录
    let total = repositories.len();
    let mut i = 0;
    while i < total {
        let repo = &repositories[i];
        let modified_str = format_time_ago(repo.last_modified);
        let language = match repo.language.as_deref() {
            Some(lang) => lang,
            None => "-",
        };
        let tags_str = if repo.tags.is_empty() {
            "-"
        } else {
            &repo.tags.join(", ")
        };

        println!(
            "{:<20} {:<15} {:<40} {:<20}",
            repo.name, language, tags_str, modified_str
        );
        i += 1;
    }
}

/// 使用默认存储，加载数据、过滤、排序并显示仓库
pub fn list(tags: Option<String>, sort: Option<String>) -> Result<()> {
    let storage = storage::default_storage()?;
    list_with_storage(&storage, tags, sort)
}

/// 接受自定义存储实现的 list 函数（便于测试时注入自定义存储）
pub fn list_with_storage<T: RepositoryStorage>(
    storage: &T,
    tags: Option<String>,
    sort: Option<String>,
) -> Result<()> {
    let repositories = storage.load()?;
    let filtered_repos = get_filtered_repositories(&repositories, &tags, &sort);
    display_repositories(&filtered_repos);
    Ok(())
}

#[cfg(test)]
mod tests_list {
    use super::*;
    use chrono::{Duration, Utc};
    use crate::repository::Repository;
    use crate::storage;
    use anyhow::Result;

    // 构造测试用的仓库数据
    fn create_test_repositories() -> Vec<Repository> {
        let now = Utc::now();
        vec![
            Repository {
                name: "repo-a".to_string(),
                path: "/path/to/repo-a".to_string(),
                tags: vec!["rust".to_string(), "cli".to_string()],
                description: "Repository A".to_string(),
                last_modified: now - Duration::days(2),
                language: Some("Rust".to_string()),
            },
            Repository {
                name: "repo-b".to_string(),
                path: "/path/to/repo-b".to_string(),
                tags: vec!["javascript".to_string(), "web".to_string()],
                description: "Repository B".to_string(),
                last_modified: now,
                language: Some("JavaScript/TypeScript".to_string()),
            },
            Repository {
                name: "repo-c".to_string(),
                path: "/path/to/repo-c".to_string(),
                tags: vec!["rust".to_string(), "web".to_string()],
                description: "Repository C".to_string(),
                last_modified: now - Duration::days(1),
                language: Some("Rust".to_string()),
            },
        ]
    }

    #[test]
    fn test_get_filtered_repositories_all() {
        let test_repos = create_test_repositories();
        let repos = get_filtered_repositories(&test_repos, &None, &None);
        assert_eq!(repos.len(), 3);
    }

    #[test]
    fn test_get_filtered_repositories_by_tags() {
        let test_repos = create_test_repositories();
        let rust_tag = Some("rust".to_string());
        let repos = get_filtered_repositories(&test_repos, &rust_tag, &None);
        assert_eq!(repos.len(), 2);
        let mut i = 0;
        while i < repos.len() {
            assert!(repos[i].tags.contains(&"rust".to_string()));
            i += 1;
        }
    }

    #[test]
    fn test_get_filtered_repositories_by_multiple_tags() {
        let test_repos = create_test_repositories();
        let rust_web_tags = Some("rust,web".to_string());
        let repos = get_filtered_repositories(&test_repos, &rust_web_tags, &None);
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].name, "repo-c");
    }

    #[test]
    fn test_get_filtered_repositories_sort_by_name() {
        let test_repos = create_test_repositories();
        let sort_option = Some("name".to_string());
        let repos = get_filtered_repositories(&test_repos, &None, &sort_option);
        assert_eq!(repos.len(), 3);
        // 按字母顺序排序
        assert_eq!(repos[0].name, "repo-a");
        assert_eq!(repos[1].name, "repo-b");
        assert_eq!(repos[2].name, "repo-c");
    }

    #[test]
    fn test_get_filtered_repositories_sort_by_modified() {
        let test_repos = create_test_repositories();
        let sort_option = Some("modified".to_string());
        let repos = get_filtered_repositories(&test_repos, &None, &sort_option);
        assert_eq!(repos.len(), 3);
        // 最近修改的在前：repo-b 最近，其次 repo-c，再次 repo-a
        assert_eq!(repos[0].name, "repo-b");
        assert_eq!(repos[1].name, "repo-c");
        assert_eq!(repos[2].name, "repo-a");
    }

    #[test]
    fn test_get_filtered_repositories_empty() {
        let empty_repos: Vec<Repository> = Vec::new();
        let repos = get_filtered_repositories(&empty_repos, &None, &None);
        assert!(repos.is_empty());
    }

    #[test]
    fn test_format_time_ago() {
        let now = Utc::now();
        assert_eq!(format_time_ago(now), "Today");
        assert_eq!(format_time_ago(now - Duration::days(1)), "Yesterday");
        assert_eq!(format_time_ago(now - Duration::days(5)), "5 days ago");
    }

    #[test]
    fn test_list_integration() {
        let test_repos = create_test_repositories();
        let storage = TestStorage {
            repositories: test_repos,
        };
        let result = list_with_storage(&storage, Some("rust".to_string()), Some("name".to_string()));
        assert!(result.is_ok());
    }

    /// 简单的测试存储实现，用于 list_integration 测试
    struct TestStorage {
        repositories: Vec<Repository>,
    }

    impl storage::RepositoryStorage for TestStorage {
        fn load(&self) -> Result<Vec<Repository>> {
            Ok(self.repositories.clone())
        }
        fn save(&self, _repositories: &[Repository]) -> Result<()> {
            Ok(())
        }
    }
}
