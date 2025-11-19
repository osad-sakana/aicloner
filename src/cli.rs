use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

const DEFAULT_CONFIG: &str = ".aicloner.toml";

#[derive(Debug, Parser)]
#[command(
    name = "aicloner",
    version,
    about = "Task ごとに git clone を管理するツール"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Add(AddArgs),
    Rm(RmArgs),
    List(ListArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long = "repo-url", value_name = "URL")]
    pub repo_url: String,
    #[arg(
        long = "workspaces-dir",
        default_value = "workspaces",
        value_name = "PATH"
    )]
    pub workspaces_dir: String,
    #[arg(long = "config", default_value = DEFAULT_CONFIG, value_name = "PATH")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    pub task_name: String,
    #[arg(long = "from", default_value = "main", value_name = "BRANCH")]
    pub base_branch: String,
    #[arg(long = "config", default_value = DEFAULT_CONFIG, value_name = "PATH")]
    pub config: PathBuf,
}

#[derive(Debug, Args)]
pub struct RmArgs {
    pub task_name: String,
    #[arg(long = "config", default_value = DEFAULT_CONFIG, value_name = "PATH")]
    pub config: PathBuf,
    #[arg(long = "force", default_value_t = false)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long = "config", default_value = DEFAULT_CONFIG, value_name = "PATH")]
    pub config: PathBuf,
}
