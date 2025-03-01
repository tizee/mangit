use anyhow::Result;
use clap::{Parser, Subcommand};
use std::process;

mod config;
mod repository;
mod storage;
mod commands;

use commands::{
    init::init,
    add::add,
    remove::remove,
    list::list,
    search::search,
    info::info,
    update::update
};

#[derive(Parser)]
#[command(name = "mangit")]
#[command(about = "Manage your Git repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize mangit
    Init,

    /// Add a repository
    Add {
        /// Path to the repository
        path: String,

        /// Name of the repository
        #[arg(short, long)]
        name: Option<String>,

        /// Tags (comma separated)
        #[arg(short, long)]
        tags: Option<String>,

        /// Description
        #[arg(short, long, value_name = "DESC")]
        desc: Option<String>,
    },

    /// Remove a repository
    Remove {
        /// Name or ID of the repository
        name_or_id: String,
    },

    /// List repositories
    List {
        /// Filter by tags (comma separated)
        #[arg(long)]
        tags: Option<String>,

        /// Sort by field
        #[arg(long)]
        sort: Option<String>,
    },

    /// Search repositories
    Search {
        /// Search query
        query: String,
    },

    /// Show repository information
    Info {
        /// Name or ID of the repository
        name_or_id: String,
    },

    /// Update repository metadata
    Update {
        /// Name or ID of the repository (optional)
        name_or_id: Option<String>,
    },
}

fn run_command(command: &Commands) -> Result<()> {
    match command {
        Commands::Init => init(),
        Commands::Add { path, name, tags, desc } => {
            add(path, name.clone(), tags.clone(), desc.clone())
        },
        Commands::Remove { name_or_id } => remove(name_or_id),
        Commands::List { tags, sort } => list(tags.clone(), sort.clone()),
        Commands::Search { query } => search(query),
        Commands::Info { name_or_id } => info(name_or_id),
        Commands::Update { name_or_id } => update(name_or_id.clone()),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match run_command(&cli.command) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests_main {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_commands() {
        // Check that CLI parser is configured correctly
        let cli = Cli::command();

        // Verify expected commands are available
        let all_commands = cli.get_subcommands().collect::<Vec<_>>();

        assert!(all_commands.iter().any(|cmd| cmd.get_name() == "init"));
        assert!(all_commands.iter().any(|cmd| cmd.get_name() == "add"));
        assert!(all_commands.iter().any(|cmd| cmd.get_name() == "remove"));
        assert!(all_commands.iter().any(|cmd| cmd.get_name() == "list"));
        assert!(all_commands.iter().any(|cmd| cmd.get_name() == "search"));
        assert!(all_commands.iter().any(|cmd| cmd.get_name() == "info"));
        assert!(all_commands.iter().any(|cmd| cmd.get_name() == "update"));
    }

    // Test parsing of different commands
    #[test]
    fn test_parse_init_command() {
        let args = vec!["mangit", "init"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Init => (), // Success
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_parse_add_command() {
        let args = vec![
            "mangit", "add", "/path/to/repo",
            "--name", "test-repo",
            "--tags", "rust,cli",
            "--desc", "Test repository"
        ];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Add { path, name, tags, desc } => {
                assert_eq!(path, "/path/to/repo");
                assert_eq!(name, Some("test-repo".to_string()));
                assert_eq!(tags, Some("rust,cli".to_string()));
                assert_eq!(desc, Some("Test repository".to_string()));
            },
            _ => panic!("Wrong command parsed"),
        }
    }

    #[test]
    fn test_parse_remove_command() {
        let args = vec!["mangit", "remove", "test-repo"];
        let cli = Cli::parse_from(args);

        match cli.command {
            Commands::Remove { name_or_id } => {
                assert_eq!(name_or_id, "test-repo");
            },
            _ => panic!("Wrong command parsed"),
        }
    }
}
