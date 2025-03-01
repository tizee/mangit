use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Config {
    pub default_projects_dir: String,
    pub mangit_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let home = home_dir().unwrap_or_else(|| PathBuf::from("~"));
        Config {
            default_projects_dir: home.to_string_lossy().to_string(),
            mangit_dir: home.join(".mangit").to_string_lossy().to_string(),
        }
    }
}

impl Config {
    /// 新建一个 Config 实例
    pub fn new(default_projects_dir: String, mangit_dir: String) -> Self {
        Config {
            default_projects_dir,
            mangit_dir,
        }
    }

    /// 返回 mangit 目录的 PathBuf
    pub fn mangit_dir_path(&self) -> PathBuf {
        PathBuf::from(self.mangit_dir.clone())
    }

    /// 返回配置文件的完整路径
    pub fn config_path(&self) -> PathBuf {
        self.mangit_dir_path().join("config.json")
    }

    /// 返回仓库文件的完整路径
    pub fn repos_path(&self) -> PathBuf {
        self.mangit_dir_path().join("repos.json")
    }

    /// 确保 mangit 目录存在，如果不存在则创建之
    pub fn ensure_mangit_dir(&self) -> Result<()> {
        let dir = self.mangit_dir_path();
        if !dir.exists() {
            fs::create_dir_all(&dir).context("Failed to create mangit directory")?;
        }
        Ok(())
    }

    /// 从配置文件加载 Config，如果文件不存在则写入默认配置后返回默认值
    pub fn load_from(&self) -> Result<Config> {
        let config_path = self.config_path();
        if !config_path.exists() {
            self.save()?;
            return Ok(self.clone());
        }
        let config_str = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;
        let config: Config = serde_json::from_str(&config_str)
            .context("Failed to parse config file")?;
        Ok(config)
    }

    /// 保存当前 Config 到配置文件中
    pub fn save(&self) -> Result<()> {
        let config_path = self.config_path();
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context("Failed to create parent directory")?;
            }
        }
        let config_str = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(config_path, config_str)
            .context("Failed to write config file")
    }
}

/// 判断给定路径是否为合法的 Git 仓库
pub fn is_git_repo(path: &str) -> Result<bool> {
    let expanded_path = shellexpand::tilde(path);
    let path = Path::new(expanded_path.as_ref());
    if !path.exists() {
        return Ok(false);
    }
    let git_dir = path.join(".git");
    Ok(git_dir.exists() && git_dir.is_dir())
}

#[cfg(test)]
mod tests_config {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.default_projects_dir.is_empty());
    }

    #[test]
    fn test_mangit_dir_path() {
        let temp_dir = tempdir().unwrap();
        let test_config = Config::new(
            temp_dir.path().to_str().unwrap().to_string(),
            temp_dir.path().join(".mangit").to_string_lossy().to_string(),
        );
        let expected_path = temp_dir.path().join(".mangit");
        assert_eq!(test_config.mangit_dir_path(), expected_path);
    }

    #[test]
    fn test_ensure_mangit_dir_creates_dir() {
        let temp_dir = tempdir().unwrap();
        let expected_dir = temp_dir.path().join(".mangit");
        let test_config = Config::new(
            temp_dir.path().to_str().unwrap().to_string(),
            expected_dir.to_string_lossy().to_string(),
        );
        assert!(!expected_dir.exists());
        let result = test_config.ensure_mangit_dir();
        assert!(result.is_ok());
        assert!(expected_dir.exists());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = tempdir().unwrap();
        let test_config = Config::new(
            "/test/projects".to_string(),
            temp_dir.path().join(".mangit").to_string_lossy().to_string(),
        );
        test_config.ensure_mangit_dir().unwrap();
        let save_result = test_config.save();
        assert!(save_result.is_ok());
        let load_result = test_config.load_from();
        assert!(load_result.is_ok());
        let loaded_config = load_result.unwrap();
        assert_eq!(loaded_config.default_projects_dir, "/test/projects");
    }

    #[test]
    fn test_is_git_repo_non_existent() {
        let non_existent_path = "/path/does/not/exist";
        let result = is_git_repo(non_existent_path);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_is_git_repo_valid() {
        let temp_dir = tempdir().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();
        let result = is_git_repo(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_is_git_repo_invalid() {
        let temp_dir = tempdir().unwrap();
        let result = is_git_repo(temp_dir.path().to_str().unwrap());
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
