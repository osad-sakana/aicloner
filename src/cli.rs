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
    Start(StartArgs),
    Issues(IssuesArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(value_name = "URL")]
    pub repo_url: String,
    #[arg(long = "base-dir", default_value = "base", value_name = "PATH")]
    pub base_dir: String,
    #[arg(long = "workspaces-dir", default_value = "ws", value_name = "PATH")]
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

#[derive(Debug, Args)]
pub struct StartArgs {
    #[arg(value_name = "ISSUE_NUMBER")]
    pub issue_number: u32,
    #[arg(long = "config", default_value = DEFAULT_CONFIG, value_name = "PATH")]
    pub config: PathBuf,
    /// Use Claude CLI (default)
    #[arg(long = "claude", group = "ai_tool")]
    pub use_claude: bool,
    /// Use Codex CLI
    #[arg(long = "codex", group = "ai_tool")]
    pub use_codex: bool,
    /// Use Gemini CLI
    #[arg(long = "gemini", group = "ai_tool")]
    pub use_gemini: bool,
}

impl StartArgs {
    /// Returns the selected AI tool (defaults to Claude)
    pub fn selected_tool(&self) -> crate::ai_tool::AiTool {
        if self.use_codex {
            crate::ai_tool::AiTool::Codex
        } else if self.use_gemini {
            crate::ai_tool::AiTool::Gemini
        } else {
            // Default to Claude (whether --claude is specified or not)
            crate::ai_tool::AiTool::Claude
        }
    }
}

#[derive(Debug, Args)]
pub struct IssuesArgs {
    #[arg(long = "config", default_value = DEFAULT_CONFIG, value_name = "PATH")]
    pub config: PathBuf,
}
