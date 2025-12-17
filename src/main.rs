mod cli;
mod config;
mod repo;
mod start;

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Parser;

use crate::{
    cli::{Cli, Commands},
    config::Config,
    repo::RepoManager,
    start::handle_start,
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
            manager.init_environment("main")?;
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
            println!("{:<12} {:<40} BRANCH", "TASK", "PATH");
            for info in tasks {
                let branch = info.branch.unwrap_or_else(|| "-".to_string());
                println!("{:<12} {:<40} {}", info.name, info.path.display(), branch);
            }
        }
        Commands::Start(args) => {
            ensure_aicloner_repo(&args.config)?;
            check_gh_installed()?;
            check_claude_installed()?;
            let manager = load_manager(&args.config)?;
            handle_start(args.issue_number, manager)?;
        }
        Commands::Issues(args) => {
            ensure_aicloner_repo(&args.config)?;
            check_gh_installed()?;
            let manager = load_manager(&args.config)?;
            list_issues(&manager)?;
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
        .rsplit(['/', ':'])
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

fn ensure_aicloner_repo(config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        bail!(
            "aicloner リポジトリではありません。{} が見つかりません。\n\
             先に 'aicloner init' を実行してください。",
            config_path.display()
        );
    }
    Ok(())
}

fn check_gh_installed() -> Result<()> {
    let output = Command::new("gh").arg("--version").output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => bail!(
            "GitHub CLI (gh) がインストールされていません。\n\
             https://cli.github.com/ からインストールしてください。"
        ),
    }
}

fn check_claude_installed() -> Result<()> {
    let output = Command::new("claude").arg("--version").output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => bail!("Claude CLI がインストールされていません。"),
    }
}

fn list_issues(manager: &RepoManager) -> Result<()> {
    let base_dir = manager.base_dir();

    let args = vec![
        "issue".to_string(),
        "list".to_string(),
        "--state".to_string(),
        "open".to_string(),
    ];

    println!("実行: gh {} (cwd: {})", args.join(" "), base_dir.display());
    let output = Command::new("gh")
        .args(&args)
        .current_dir(&base_dir)
        .output()
        .context("gh issue list の実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Issue一覧の取得に失敗しました: {}", stderr.trim());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    Ok(())
}
