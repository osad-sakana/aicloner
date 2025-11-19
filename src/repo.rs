use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};

use crate::config::Config;

pub struct RepoManager {
    pub config: Config,
    pub config_path: PathBuf,
}

impl RepoManager {
    pub fn new(config: Config, config_path: PathBuf) -> Self {
        Self {
            config,
            config_path,
        }
    }

    pub fn base_dir(&self) -> PathBuf {
        self.resolve_path(&self.config.base_dir)
    }

    pub fn workspaces_dir(&self) -> PathBuf {
        self.resolve_path(&self.config.workspaces_dir)
    }

    fn resolve_path(&self, relative: &str) -> PathBuf {
        let base = self
            .config_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        base.join(relative)
    }

    pub fn init_base_repo(&self) -> Result<()> {
        let base_dir = self.base_dir();
        // base ディレクトリの状態に応じて clone または fetch を実行
        if !base_dir.exists() {
            if let Some(parent) = base_dir.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "ベースレポジトリ親ディレクトリの作成に失敗しました: {}",
                        parent.display()
                    )
                })?;
            }
            let base_dir_str = base_dir.display().to_string();
            let args = vec![
                "clone".to_string(),
                self.config.repo_url.clone(),
                base_dir_str.clone(),
            ];
            run_command("git", &args, None)?;
        } else {
            let base_dir_str = base_dir.display().to_string();
            let args = vec![
                "-C".to_string(),
                base_dir_str.clone(),
                "fetch".to_string(),
                "--all".to_string(),
            ];
            run_command("git", &args, None)?;
        }

        let workspaces_dir = self.workspaces_dir();
        fs::create_dir_all(&workspaces_dir).with_context(|| {
            format!(
                "ワークスペースディレクトリの作成に失敗しました: {}",
                workspaces_dir.display()
            )
        })?;
        Ok(())
    }

    pub fn create_task_clone(&self, task_name: &str, base_branch: &str) -> Result<()> {
        let base_dir = self.base_dir();
        if !base_dir.exists() {
            bail!(
                "ベースレポジトリが存在しません。先に init を実行してください: {}",
                base_dir.display()
            );
        }
        let workspaces_dir = self.workspaces_dir();
        if !workspaces_dir.exists() {
            bail!(
                "ワークスペースディレクトリが存在しません。先に init を実行してください: {}",
                workspaces_dir.display()
            );
        }

        let workspace_dir = workspaces_dir.join(task_name);
        if workspace_dir.exists() {
            bail!(
                "タスク \"{}\" は既に存在します: {}",
                task_name,
                workspace_dir.display()
            );
        }

        let repo_dir_str = workspace_dir.display().to_string();
        let base_dir_str = base_dir.display().to_string();

        let reference_args = vec![
            "clone".to_string(),
            "--reference".to_string(),
            base_dir_str.clone(),
            self.config.repo_url.clone(),
            repo_dir_str.clone(),
        ];

        // --reference を使った clone が失敗した場合は通常 clone に切り替える
        if let Err(err) = run_command("git", &reference_args, None) {
            eprintln!("参照付き clone に失敗したためリモート clone にフォールバックします: {err}");
            if workspace_dir.exists() {
                let _ = fs::remove_dir_all(&workspace_dir);
            }
            let fallback_args = vec![
                "clone".to_string(),
                self.config.repo_url.clone(),
                repo_dir_str.clone(),
            ];
            run_command("git", &fallback_args, None)?;
        }

        let checkout_args = vec![
            "-C".to_string(),
            repo_dir_str.clone(),
            "checkout".to_string(),
            base_branch.to_string(),
        ];
        run_command("git", &checkout_args, None)?;
        println!(
            "タスク \"{}\" の clone を作成しました: {}",
            task_name,
            workspace_dir.display()
        );
        Ok(())
    }

    pub fn remove_task_clone(&self, task_name: &str, force: bool) -> Result<()> {
        let workspace_dir = self.workspaces_dir().join(task_name);
        if !workspace_dir.exists() {
            bail!("タスク \"{}\" は存在しません。", task_name);
        }

        if !force {
            // y が入力された場合のみ削除を実行
            let prompt = format!(
                "Remove workspace \"{}\" at \"{}\"? [y/N]: ",
                task_name,
                workspace_dir.display()
            );
            print!("{prompt}");
            io::stdout()
                .flush()
                .context("プロンプトの表示に失敗しました")?;
            let mut answer = String::new();
            io::stdin()
                .read_line(&mut answer)
                .context("入力の読み取りに失敗しました")?;
            let answer = answer.trim().to_lowercase();
            if answer != "y" {
                println!("削除を中止しました。");
                return Ok(());
            }
        }

        fs::remove_dir_all(&workspace_dir).with_context(|| {
            format!(
                "ディレクトリの削除に失敗しました: {}",
                workspace_dir.display()
            )
        })?;
        println!(
            "タスク \"{}\" のワークスペースを削除しました: {}",
            task_name,
            workspace_dir.display()
        );
        Ok(())
    }

    pub fn list_tasks(&self) -> Result<Vec<TaskInfo>> {
        let workspaces_dir = self.workspaces_dir();
        if !workspaces_dir.exists() {
            bail!(
                "ワークスペースディレクトリが存在しません: {}",
                workspaces_dir.display()
            );
        }
        let mut tasks = Vec::new();
        for entry in fs::read_dir(&workspaces_dir).with_context(|| {
            format!(
                "ワークスペース一覧の取得に失敗しました: {}",
                workspaces_dir.display()
            )
        })? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            // タスクディレクトリ自体が clone 先
            let branch = if path.exists() {
                let repo_str = path.display().to_string();
                let args = vec![
                    "-C".to_string(),
                    repo_str.clone(),
                    "rev-parse".to_string(),
                    "--abbrev-ref".to_string(),
                    "HEAD".to_string(),
                ];
                match run_command_capture("git", &args, None) {
                    Ok(output) => Some(output.trim().to_string()),
                    Err(_) => None,
                }
            } else {
                None
            };

            tasks.push(TaskInfo {
                name,
                path: path.clone(),
                branch,
            });
        }
        tasks.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tasks)
    }
}

pub struct TaskInfo {
    pub name: String,
    pub path: PathBuf,
    pub branch: Option<String>,
}

fn run_command(program: &str, args: &[String], dir: Option<&Path>) -> Result<()> {
    // 外部コマンドを実行し、失敗時は anyhow::Error にラップ
    log_command(program, args, dir);
    let mut command = Command::new(program);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.args(args);
    let status = command
        .status()
        .with_context(|| format!("コマンドの起動に失敗しました: {}", program))?;
    if !status.success() {
        bail!("コマンドが失敗しました: {} {}", program, join_args(args));
    }
    Ok(())
}

fn run_command_capture(program: &str, args: &[String], dir: Option<&Path>) -> Result<String> {
    // stdout を取得する場合はこちらを利用
    log_command(program, args, dir);
    let mut command = Command::new(program);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    command.args(args);
    let output = command
        .output()
        .with_context(|| format!("コマンドの起動に失敗しました: {}", program))?;
    if !output.status.success() {
        bail!("コマンドが失敗しました: {} {}", program, join_args(args));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn log_command(program: &str, args: &[String], dir: Option<&Path>) {
    let joined = args.join(" ");
    match dir {
        Some(d) => println!("実行: {} {} (cwd: {})", program, joined, d.display()),
        None => println!("実行: {} {}", program, joined),
    }
}

fn join_args(args: &[String]) -> String {
    args.join(" ")
}
