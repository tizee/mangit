use anyhow::Result;
use crate::config::Config;
use crate::storage;
use crate::storage::RepositoryStorage;

/// 使用指定的 Config 初始化 mangit：
/// 1. 检查并创建 mangit 目录；
/// 2. 保存配置文件；
/// 3. 初始化仓库存储（生成空的 repos.json）。
pub fn init_with_config(config: &Config) -> Result<()> {
    config.ensure_mangit_dir()?;
    config.save()?;
    let storage = storage::FileStorage::from_config(config)?;
    storage.save(&Vec::new());
    println!("Initialized mangit at {}", config.mangit_dir_path().display());
    println!("Your repositories will be tracked here.");
    Ok(())
}

/// 使用默认配置进行初始化
pub fn init() -> Result<()> {
    let config = Config::default();
    init_with_config(&config)
}

#[cfg(test)]
mod tests_init {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use tempfile::tempdir;
    use serde_json;

    #[test]
    fn test_init_with_config_creates_necessary_files() {
        // 使用临时目录构造自定义配置
        let temp_dir = tempdir().unwrap();
        let expected_dir = temp_dir.path().join(".mangit");
        let expected_config = expected_dir.join("config.json");
        let expected_repos = expected_dir.join("repos.json");

        let custom_config = Config::new(
            temp_dir.path().to_str().unwrap().to_string(),
            expected_dir.to_string_lossy().to_string(),
        );

        // 执行初始化
        let result = init_with_config(&custom_config);
        assert!(result.is_ok());
        assert!(expected_dir.exists());
        assert!(expected_config.exists());
        assert!(expected_repos.exists());

        // 验证配置文件内容
        let config_content = fs::read_to_string(&expected_config).unwrap();
        let loaded_config: Config = serde_json::from_str(&config_content).unwrap();
        assert!(!loaded_config.default_projects_dir.is_empty());
    }
}
