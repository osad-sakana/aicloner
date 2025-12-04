mod cli;
mod config;
mod repo;

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::Parser;

use crate::{
    cli::{Cli, Commands},
    config::Config,
    repo::RepoManager,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => {
            let repo_name = repo_dir_name(&args.repo_url)?;
            let repo_root = std::env::current_dir()?.join(&repo_name);
            if repo_root.exists() {
                bail!(
                    "同名のディレクトリが既に存在します: {}",
                    repo_root.display()
                );
            }

            let config_path = resolve_config_path(&repo_root, &args.config);
            let config = Config {
                repo_url: args.repo_url,
                base_dir: args.base_dir,
                workspaces_dir: args.workspaces_dir,
            };
            config.save(&config_path)?;
            let manager = RepoManager::new(config, config_path.clone());
            manager.init_environment()?;
            println!("初期化が完了しました: {}", repo_root.display());
        }
        Commands::Add(args) => {
            let manager = load_manager(&args.config)?;
            manager.create_task_clone(&args.task_name, &args.base_branch)?;
        }
        Commands::Rm(args) => {
            let manager = load_manager(&args.config)?;
            manager.remove_task_clone(&args.task_name, args.force)?;
        }
        Commands::List(args) => {
            let manager = load_manager(&args.config)?;
            let tasks = manager.list_tasks()?;
            println!("{:<12} {:<40} {}", "TASK", "PATH", "BRANCH");
            for info in tasks {
                let branch = info.branch.unwrap_or_else(|| "-".to_string());
                println!("{:<12} {:<40} {}", info.name, info.path.display(), branch);
            }
        }
    }

    Ok(())
}

fn load_manager(path: &Path) -> Result<RepoManager> {
    let config = Config::load(path)?;
    Ok(RepoManager::new(config, path.to_path_buf()))
}

fn resolve_config_path(repo_root: &Path, config: &PathBuf) -> PathBuf {
    if config.is_absolute() {
        config.clone()
    } else {
        repo_root.join(config)
    }
}

fn repo_dir_name(repo_url: &str) -> Result<String> {
    let trimmed = repo_url.trim_end_matches('/');
    let name_part = trimmed
        .rsplit(|c| c == '/' || c == ':')
        .next()
        .unwrap_or("");
    let name = name_part
        .strip_suffix(".git")
        .unwrap_or(name_part)
        .to_string();
    if name.is_empty() {
        bail!("リポジトリ名を URL から抽出できませんでした: {}", repo_url);
    }
    Ok(name)
}
