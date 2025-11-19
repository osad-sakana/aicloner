mod cli;
mod config;
mod repo;

use std::path::Path;

use anyhow::Result;
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
            let config_path = args.config;
            let config = Config {
                repo_url: args.repo_url,
                base_dir: args.base_dir,
                workspaces_dir: args.workspaces_dir,
            };
            // init は設定ファイルを書いたあとベースレポジトリを準備
            config.save(&config_path)?;
            let manager = RepoManager::new(config, config_path.clone());
            manager.init_base_repo()?;
            println!("設定を {} に保存しました。", config_path.display());
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
    // 設定ファイルの場所を起点に相対パスを解決する
    Ok(RepoManager::new(config, path.to_path_buf()))
}
