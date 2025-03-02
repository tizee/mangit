mod config;
mod repository;
mod storage;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use config::Config;
use storage::Storage;

#[derive(Parser, Debug)]
#[clap(author, version, about = "Manage Git repositories with tags")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize mangit
    Init,

    /// Add a repo with tags
    Add {
        /// Path to repository
        path: String,

        /// Tags for the repository (comma separated)
        #[clap(short, long)]
        tags: String,
    },

    /// Delete a repo
    Delete {
        /// Path to repository
        path: String,
    },

    /// Update a repo's tags
    Update {
        /// Path to repository
        path: String,

        /// New tags for the repository (comma separated)
        #[clap(short, long)]
        tags: String,
    },

    /// Search for repos by tag or multiple tags
    Search {
        /// Tag(s) to search for (comma separated)
        tags: String,
    },

    /// Access a repo (updates frecency)
    Access {
        /// Path to repository
        path: String,
    },

    /// Reset frequency data for a repo or all repos
    Reset {
        /// Path to repository (if not provided, resets all repos)
        #[clap(short, long)]
        path: Option<String>,
    },

    /// List all tags with their usage counts
    Tags,
}

fn parse_tags(tags_str: &str) -> Vec<String> {
    tags_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::default();

    match cli.command {
        Commands::Init => {
            config.ensure_mangit_dir()?;
            let storage = Storage::new(&config)?;
            storage.save(&config)?;
            println!("Initialized mangit at {}", config.mangit_dir);
            Ok(())
        }

        Commands::Add { path, tags } => {
            let mut storage = Storage::new(&config)?;
            let tags = parse_tags(&tags);

            match storage.add_repo(&path, tags) {
                Ok(true) => {
                    println!("Added repo: {}", path);
                    storage.save(&config)?;
                    Ok(())
                }
                Ok(false) => {
                    println!("Updated existing repo: {}", path);
                    storage.save(&config)?;
                    Ok(())
                }
                Err(e) => Err(anyhow!("Failed to add repo: {}", e)),
            }
        }

        Commands::Delete { path } => {
            let mut storage = Storage::new(&config)?;

            match storage.delete_repo(&path) {
                Ok(true) => {
                    println!("Deleted repo: {}", path);
                    storage.save(&config)?;
                    Ok(())
                }
                Ok(false) => Err(anyhow!("Repo not found: {}", path)),
                Err(e) => Err(anyhow!("Failed to delete repo: {}", e)),
            }
        }

        Commands::Update { path, tags } => {
            let mut storage = Storage::new(&config)?;
            let tags = parse_tags(&tags);

            match storage.update_repo(&path, tags) {
                Ok(true) => {
                    println!("Updated repo: {}", path);
                    storage.save(&config)?;
                    Ok(())
                }
                Ok(false) => Err(anyhow!("Repo not found: {}", path)),
                Err(e) => Err(anyhow!("Failed to update repo: {}", e)),
            }
        }

        Commands::Search { tags } => {
            let mut storage = Storage::new(&config)?;
            let tag_list = parse_tags(&tags);

            if tag_list.is_empty() {
                println!("No tags specified for search");
                return Ok(());
            }

            let matches = storage.search_by_tags(&tag_list);

            if matches.is_empty() {
                if tag_list.len() == 1 {
                    println!("No repos found with tag: {}", tag_list[0]);
                } else {
                    println!("No repos found with all tags: {}", tags);
                }
            } else {
                // Simple output, one path per line for easy integration with tools like fzf
                for path in matches {
                    println!("{}", path);
                }
                // Save after search to update frecency data
                storage.save(&config)?;
            }

            Ok(())
        }

        Commands::Access { path } => {
            let mut storage = Storage::new(&config)?;

            match storage.record_access(&path) {
                Ok(true) => {
                    storage.save(&config)?;
                    Ok(())
                }
                Ok(false) => Err(anyhow!("Repo not found: {}", path)),
                Err(e) => Err(anyhow!("Failed to access repo: {}", e)),
            }
        }

        Commands::Reset { path } => {
            let mut storage = Storage::new(&config)?;

            match storage.reset_frequency(path.as_deref()) {
                Ok(count) => {
                    if let Some(p) = path {
                        if count > 0 {
                            println!("Repo not found: {}", p);
                        }
                    } else {
                        println!("Reset frequency for {} repos", count);
                    }
                    storage.save(&config)?;
                    Ok(())
                }
                Err(e) => Err(anyhow!("Failed to reset frequency: {}", e)),
            }
        }

        Commands::Tags => {
            let storage = Storage::new(&config)?;
            let all_tags = storage.get_all_tags();

            if all_tags.is_empty() {
                println!("No tags found in any repositories");
                return Ok(());
            }

            // Convert to sorted vec for consistent output
            let mut tag_counts: Vec<(String, usize)> = all_tags.into_iter().collect();
            tag_counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

            println!("All tags (tag: count):");
            for (tag, count) in tag_counts {
                println!("{}: {}", tag, count);
            }

            Ok(())
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests_main {
    use super::*;

    #[test]
    fn test_parse_tags() {
        let tags = parse_tags("rust,cli,tool");
        assert_eq!(tags, vec!["rust", "cli", "tool"]);

        // Test with spaces
        let tags = parse_tags("rust, cli, tool");
        assert_eq!(tags, vec!["rust", "cli", "tool"]);

        // Test with empty parts
        let tags = parse_tags("rust,,cli");
        assert_eq!(tags, vec!["rust", "cli"]);

        // Test with empty string
        let tags = parse_tags("");
        assert_eq!(tags.len(), 0);
    }
}
