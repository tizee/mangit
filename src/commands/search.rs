use anyhow::Result;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::repository::Repository;
use crate::storage;
use crate::storage::RepositoryStorage;
use std::cmp::Ordering;

/// 比较两个仓库的分数，分数高的排在前面
fn compare_scores(a: &(Repository, i64), b: &(Repository, i64)) -> Ordering {
    b.1.cmp(&a.1)
}

/// 搜索仓库，返回仓库及匹配分数的列表（纯函数，无副作用）
pub fn search_repositories(repositories: &[Repository], query: &str) -> Vec<(Repository, i64)> {
    let mut results: Vec<(Repository, i64)> = Vec::new();

    // 如果查询为空，则直接返回所有仓库，分数均为 0
    if query.is_empty() {
        let len = repositories.len();
        let mut index = 0;
        while index < len {
            results.push((repositories[index].clone(), 0));
            index += 1;
        }
        return results;
    }

    let matcher = SkimMatcherV2::default();
    let len = repositories.len();
    let mut index = 0;
    while index < len {
        let repo = &repositories[index];
        // 基本匹配：检查查询是否出现在仓库名称、标签或描述中
        let basic_match = repo.matches_query(query);
        // 模糊匹配：对名称进行匹配
        let score = match matcher.fuzzy_match(&repo.name, query) {
            Some(s) => s,
            None => 0,
        };

        if basic_match || score > 0 {
            results.push((repo.clone(), score));
        }
        index += 1;
    }

    results.sort_by(compare_scores);
    results
}

/// 将仓库搜索结果以表格形式打印出来
fn display_search_results(matches: &[(Repository, i64)], query: &str) {
    if matches.is_empty() {
        if query.is_empty() {
            println!("No repositories found");
        } else {
            println!("No matching repositories found for query: '{}'", query);
        }
        return;
    }

    // 打印表头
    println!("{:<20} {:<15} {:<40}", "NAME", "LANGUAGE", "TAGS");
    println!("{}", "-".repeat(75));

    let len = matches.len();
    let mut index = 0;
    while index < len {
        let (ref repo, _) = matches[index];
        // 语言字段处理
        let language = match repo.language.as_deref() {
            Some(lang) => lang,
            None => "-",
        };
        // 手动拼接标签字符串，避免使用闭包或语法糖
        let tags_str: String;
        if repo.tags.is_empty() {
            tags_str = "-".to_string();
        } else {
            let mut result = String::new();
            let tag_len = repo.tags.len();
            let mut tag_index = 0;
            while tag_index < tag_len {
                result.push_str(&repo.tags[tag_index]);
                if tag_index < tag_len - 1 {
                    result.push_str(", ");
                }
                tag_index += 1;
            }
            tags_str = result;
        }

        println!("{:<20} {:<15} {:<40}", repo.name, language, tags_str);
        index += 1;
    }
}

/// 使用给定的存储接口进行仓库搜索，并显示结果
pub fn search_with_storage<S>(storage: &S, query: &str) -> Result<()>
where
    S: RepositoryStorage,
{
    let repositories = storage.load()?;
    let matches = search_repositories(&repositories, query);
    display_search_results(&matches, query);
    Ok(())
}

/// 主搜索函数，使用默认存储
pub fn search(query: &str) -> Result<()> {
    let storage = storage::default_storage()?;
    search_with_storage(&storage, query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use anyhow::Result;

    // 构造用于测试的仓库数据
    fn create_test_repositories() -> Vec<Repository> {
        vec![
            Repository {
                name: "rust-cli".to_string(),
                path: "/path/to/rust-cli".to_string(),
                tags: vec!["rust".to_string(), "cli".to_string()],
                description: "A command-line interface in Rust".to_string(),
                last_modified: Utc::now(),
                language: Some("Rust".to_string()),
            },
            Repository {
                name: "web-app".to_string(),
                path: "/path/to/web-app".to_string(),
                tags: vec!["javascript".to_string(), "web".to_string()],
                description: "A web application".to_string(),
                last_modified: Utc::now(),
                language: Some("JavaScript/TypeScript".to_string()),
            },
            Repository {
                name: "data-analysis".to_string(),
                path: "/path/to/data-analysis".to_string(),
                tags: vec!["python".to_string(), "data-science".to_string()],
                description: "Data analysis scripts".to_string(),
                last_modified: Utc::now(),
                language: Some("Python".to_string()),
            },
        ]
    }

    #[test]
    fn test_search_repositories_matching_name() {
        let test_repos = create_test_repositories();
        let matches = search_repositories(&test_repos, "rust");

        let mut found_rust_cli = false;
        let mut found_web_app = false;
        let len = matches.len();
        let mut index = 0;
        while index < len {
            let (ref repo, _) = matches[index];
            if repo.name == "rust-cli" {
                found_rust_cli = true;
            }
            if repo.name == "web-app" {
                found_web_app = true;
            }
            index += 1;
        }
        assert!(found_rust_cli);
        assert!(!found_web_app);
    }

    #[test]
    fn test_search_repositories_matching_tags() {
        let test_repos = create_test_repositories();
        let matches = search_repositories(&test_repos, "web");

        let mut found_web_app = false;
        let len = matches.len();
        let mut index = 0;
        while index < len {
            let (ref repo, _) = matches[index];
            if repo.name == "web-app" {
                found_web_app = true;
            }
            index += 1;
        }
        assert!(found_web_app);
    }

    #[test]
    fn test_search_repositories_matching_description() {
        let test_repos = create_test_repositories();
        let matches = search_repositories(&test_repos, "analysis");

        let mut found_data_analysis = false;
        let len = matches.len();
        let mut index = 0;
        while index < len {
            let (ref repo, _) = matches[index];
            if repo.name == "data-analysis" {
                found_data_analysis = true;
            }
            index += 1;
        }
        assert!(found_data_analysis);
    }

    #[test]
    fn test_search_repositories_no_matches() {
        let test_repos = create_test_repositories();
        let matches = search_repositories(&test_repos, "nonexistent");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_search_repositories_empty_query() {
        let test_repos = create_test_repositories();
        let matches = search_repositories(&test_repos, "");

        // 应返回所有仓库，且分数均为 0
        assert_eq!(matches.len(), test_repos.len());
        let len = matches.len();
        let mut index = 0;
        while index < len {
            let (_, score) = &matches[index];
            assert_eq!(*score, 0);
            index += 1;
        }
    }

    #[test]
    fn test_search_repositories_empty_repositories() {
        let empty_repos: Vec<Repository> = Vec::new();
        let matches = search_repositories(&empty_repos, "any");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_search_integration() {
        let test_repos = create_test_repositories();
        let test_storage = TestStorage {
            repositories: test_repos.clone(),
        };
        let result = search_with_storage(&test_storage, "rust");
        assert!(result.is_ok());
    }

    // 用于集成测试的存储实现
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
