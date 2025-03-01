use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub name: String,
    pub path: String,
    pub tags: Vec<String>,
    pub description: String,
    pub last_modified: DateTime<Utc>,
    pub language: Option<String>,
}

impl Repository {
    pub fn new(
        name: String,
        path: String,
        tags: Vec<String>,
        description: String,
    ) -> Self {
        Repository {
            name,
            path,
            tags,
            description,
            last_modified: Utc::now(),
            language: None,
        }
    }

    pub fn detect_language(&mut self) {
        let path = Path::new(&self.path);

        // Check for common project files to determine language
        if path.join("Cargo.toml").exists() {
            self.language = Some("Rust".to_string());
        } else if path.join("package.json").exists() {
            self.language = Some("JavaScript/TypeScript".to_string());
        } else if path.join("go.mod").exists() {
            self.language = Some("Go".to_string());
        } else if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            self.language = Some("Java".to_string());
        } else if path.join("requirements.txt").exists() || path.join("setup.py").exists() {
            self.language = Some("Python".to_string());
        } else if path.join("CMakeLists.txt").exists() {
            self.language = Some("C/C++".to_string());
        }
        // More language detection can be added here
    }

    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }

        let query_lower = query.to_lowercase();
        let tokens: Vec<&str> = query_lower.split_whitespace().collect();

        for token in tokens {
            if self.name.to_lowercase().contains(token) {
                return true;
            }

            if self.description.to_lowercase().contains(token) {
                return true;
            }

            if self.tags.iter().any(|tag| tag.to_lowercase().contains(token)) {
                return true;
            }

            if let Some(lang) = &self.language {
                if lang.to_lowercase().contains(token) {
                    return true;
                }
            }
        }

        false
    }

    pub fn matches_tags(&self, tags: &[String]) -> bool {
        if tags.is_empty() {
            return true;
        }

        for tag in tags {
            if !self.tags.iter().any(|t| t == tag) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests_repository {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_new_repository() {
        // Arrange
        let name = "test-repo".to_string();
        let path = "/path/to/repo".to_string();
        let tags = vec!["rust".to_string(), "cli".to_string()];
        let description = "A test repository".to_string();

        // Act
        let repo = Repository::new(name.clone(), path.clone(), tags.clone(), description.clone());

        // Assert
        assert_eq!(repo.name, name);
        assert_eq!(repo.path, path);
        assert_eq!(repo.tags, tags);
        assert_eq!(repo.description, description);
        assert!(repo.language.is_none());
    }

    #[test]
    fn test_detect_language_rust() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path().to_str().unwrap().to_string();

        // Create a Cargo.toml file
        let cargo_path = temp_dir.path().join("Cargo.toml");
        let mut cargo_file = fs::File::create(cargo_path).unwrap();
        writeln!(cargo_file, "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();

        // Create a repository
        let mut repo = Repository::new(
            "test-repo".to_string(),
            repo_path.clone(),
            Vec::new(),
            "".to_string(),
        );

        // Act
        repo.detect_language();

        // Assert
        assert_eq!(repo.language, Some("Rust".to_string()));
    }

    #[test]
    fn test_detect_language_javascript() {
        let temp_dir = tempdir().unwrap();
        let repo_path = temp_dir.path().to_str().unwrap().to_string();

        // Create a package.json file
        let pkg_path = temp_dir.path().join("package.json");
        let mut pkg_file = fs::File::create(pkg_path).unwrap();
        writeln!(pkg_file, "{{\"name\": \"test\", \"version\": \"1.0.0\"}}").unwrap();

        // Create a repository
        let mut repo = Repository::new(
            "test-repo".to_string(),
            repo_path.clone(),
            Vec::new(),
            "".to_string(),
        );

        // Act
        repo.detect_language();

        // Assert
        assert_eq!(repo.language, Some("JavaScript/TypeScript".to_string()));
    }

    #[test]
    fn test_matches_query_empty() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(repo.matches_query(""));
    }

    #[test]
    fn test_matches_query_name() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(repo.matches_query("test"));
        assert!(repo.matches_query("repo"));
        assert!(repo.matches_query("TEST")); // Case insensitive
    }

    #[test]
    fn test_matches_query_description() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(repo.matches_query("repository"));
    }

    #[test]
    fn test_matches_query_tags() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(repo.matches_query("rust"));
        assert!(repo.matches_query("CLI")); // Case insensitive
    }

    #[test]
    fn test_matches_query_language() {
        // Arrange
        let mut repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );
        repo.language = Some("Rust".to_string());

        // Act & Assert
        assert!(repo.matches_query("rust"));
    }

    #[test]
    fn test_matches_tags_empty() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(repo.matches_tags(&[]));
    }

    #[test]
    fn test_matches_tags_single() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(repo.matches_tags(&["rust".to_string()]));
        assert!(repo.matches_tags(&["cli".to_string()]));
    }

    #[test]
    fn test_matches_tags_multiple() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(repo.matches_tags(&["rust".to_string(), "cli".to_string()]));
    }

    #[test]
    fn test_matches_tags_not_found() {
        // Arrange
        let repo = Repository::new(
            "test-repo".to_string(),
            "/path/to/repo".to_string(),
            vec!["rust".to_string(), "cli".to_string()],
            "A test repository".to_string(),
        );

        // Act & Assert
        assert!(!repo.matches_tags(&["web".to_string()]));
        assert!(!repo.matches_tags(&["rust".to_string(), "web".to_string()]));
    }
}
