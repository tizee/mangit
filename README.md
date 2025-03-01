# mangit - Semantic Git Repository Manager

`mangit` is a command-line tool that helps you manage, search, and navigate your local Git repositories using tags and metadata rather than relying solely on directory structure.

![License](https://img.shields.io/badge/license-MIT-blue.svg)

## Motivation

- Inspired by the need to manage many repositories across different project domains
- Built with Rust for reliability and performance

## 🌟 Features

- **Tag-based organization**: Categorize repositories with custom tags
- **Semantic search**: Find repositories based on names, tags, or descriptions
- **Smart navigation**: Quickly open and navigate to repositories
- **Frecency-based sorting**: Results ordered by frequency and recency of use
- **Zero changes to your existing workflow**: Works with your current directory structure

## 🚀 Installation

### Prerequisites

- Rust and Cargo (for building from source)
- Git
- fzf (optional, for interactive features)

### From Source

```bash
# Clone the repository
git clone https://github.com/username/mangit.git
cd mangit

# Build and install
cargo install --path .
```

## 📝 Quick Start

```bash
# Initialize mangit
mangit init

# Add your first repository
mangit add ~/projects/some-repo --tags "rust,cli,tool"

# Search for repositories by tag
mangit search "rust"

# Access a repository (records usage for frecency)
mangit access ~/projects/some-repo
```

## 🛠️ Commands

| Command | Description |
|---------|-------------|
| `mangit init` | Initialize mangit |
| `mangit add <path> --tags <tags>` | Add a repository |
| `mangit delete <path>` | Remove a repository from mangit |
| `mangit update <path> --tags <tags>` | Update repository tags |
| `mangit search <tag>` | Search for repositories by tag |
| `mangit access <path>` | Record repository access (for frecency) |
| `mangit reset [--path <path>]` | Reset frequency data for one or all repos |

## 🔌 Shell Integration

### ZSH Integration

mangit comes with a ZSH integration script that provides enhanced functionality:

1. Save the [mangit.zsh](./mangit.zsh) script to a location like `~/.config/zsh/mangit.zsh`
2. Add this to your `~/.zshrc`:
   ```zsh
   source ~/.config/zsh/mangit.zsh
   ```

The integration provides these commands:

| Command | Description |
|---------|-------------|
| `mgcd [query]` | Navigate to a repository with fuzzy search |
| `mgadd [path]` | Add current/specified directory as repository |
| `mgl` | List repositories |
| `mgs <tag>` | Search repositories by tag |
| `mgpull <tag>` | Pull all repositories with tag |
| `mgstatus <tag>` | Check status of repositories with tag |
| `mgdash` | Interactive dashboard |

## 🔍 Use Cases

### Managing Multiple Projects

```bash
# Add several projects with descriptive tags
mangit add ~/work/backend --tags "work,backend,rust"
mangit add ~/work/frontend --tags "work,frontend,react"
mangit add ~/personal/blog --tags "personal,web,hugo"

# Find all work projects
mangit search "work"

# Find all Rust projects
mangit search "rust"
```

### Task-based Workflow

```bash
# Tag repositories by feature or task
mangit update ~/work/backend --tags "work,backend,rust,auth-feature"
mangit update ~/work/frontend --tags "work,frontend,react,auth-feature"

# Find all repos related to the auth feature
mangit search "auth-feature"
```

### With ZSH Integration

```bash
# Navigate to a Rust project
mgcd rust

# Pull all work repositories
mgpull work

# Check status of all auth-feature repositories
mgstatus auth-feature
```

## 🔄 How it Works

mangit stores metadata about your Git repositories in a simple JSON file located at `~/.mangit/repos.json`. This metadata includes:

- Repository path
- Associated tags
- Access history (for frecency-based sorting)

The tool doesn't modify your repositories or require any changes to your existing directory structure.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.
