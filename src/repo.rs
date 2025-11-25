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

    pub fn init_environment(&self) -> Result<()> {
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

        let clone_args = vec![
            "clone".to_string(),
            "--branch".to_string(),
            base_branch.to_string(),
            "--single-branch".to_string(),
            self.config.repo_url.clone(),
            repo_dir_str.clone(),
        ];
        run_command("git", &clone_args, None).map_err(|err| {
            if workspace_dir.exists() {
                let _ = fs::remove_dir_all(&workspace_dir);
            }
            err
        })?;

        let branch_args = vec![
            "-C".to_string(),
            repo_dir_str.clone(),
            "checkout".to_string(),
            "-b".to_string(),
            task_name.to_string(),
        ];
        run_command("git", &branch_args, None)?;
        println!(
            "タスク \"{}\" 用のワークスペースとブランチ \"{}\" を作成しました: {}",
            task_name,
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
    let output = command
        .output()
        .with_context(|| format!("コマンドの起動に失敗しました: {}", program))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "コマンドが失敗しました: {} {}\nstatus: {}\nstderr: {}",
            program,
            join_args(args),
            output.status,
            stderr
        );
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
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "コマンドが失敗しました: {} {}\nstatus: {}\nstderr: {}",
            program,
            join_args(args),
            output.status,
            stderr
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn create_clone_and_list_returns_branch() -> Result<()> {
        let tmp = TempDir::new()?;
        let remote = init_remote_repo(&tmp)?;

        let config_path = tmp.path().join(".aicloner.toml");
        let config = Config {
            repo_url: remote.to_string_lossy().to_string(),
            workspaces_dir: "workspaces".to_string(),
        };
        let manager = RepoManager::new(config, config_path);
        manager.init_environment()?;

        manager.create_task_clone("task-a", "main")?;

        let workspace = manager.workspaces_dir().join("task-a");
        assert!(workspace.exists());
        let branch = current_branch(&workspace)?;
        assert_eq!(branch, "task-a");

        let tasks = manager.list_tasks()?;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "task-a");
        assert_eq!(tasks[0].branch.as_deref(), Some("task-a"));
        Ok(())
    }

    #[test]
    fn clone_failure_shows_stderr_and_cleans_directory() -> Result<()> {
        let tmp = TempDir::new()?;
        let remote = init_remote_repo(&tmp)?;

        let config_path = tmp.path().join(".aicloner.toml");
        let config = Config {
            repo_url: remote.to_string_lossy().to_string(),
            workspaces_dir: "workspaces".to_string(),
        };
        let manager = RepoManager::new(config, config_path);
        manager.init_environment()?;

        let result = manager.create_task_clone("broken", "no-such-branch");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("stderr"));
        assert!(msg.contains("fatal"));
        assert!(!manager.workspaces_dir().join("broken").exists());
        Ok(())
    }

    fn init_remote_repo(tmp: &TempDir) -> Result<PathBuf> {
        let remote = tmp.path().join("remote.git");
        run_git(&["init", "--bare", remote.to_string_lossy().as_ref()], None)?;

        let seed = tmp.path().join("seed");
        fs::create_dir_all(&seed)?;
        run_git(&["init"], Some(&seed))?;
        run_git(&["config", "user.name", "tester"], Some(&seed))?;
        run_git(&["config", "user.email", "tester@example.com"], Some(&seed))?;

        let mut file = fs::File::create(seed.join("README.md"))?;
        writeln!(file, "hello")?;
        run_git(&["add", "README.md"], Some(&seed))?;
        run_git(&["commit", "-m", "init"], Some(&seed))?;
        run_git(&["branch", "-M", "main"], Some(&seed))?;
        run_git(
            &["remote", "add", "origin", remote.to_string_lossy().as_ref()],
            Some(&seed),
        )?;
        run_git(&["push", "origin", "main"], Some(&seed))?;
        Ok(remote)
    }

    fn current_branch(repo: &Path) -> Result<String> {
        capture_git(&["rev-parse", "--abbrev-ref", "HEAD"], Some(repo))
    }

    fn run_git(args: &[&str], dir: Option<&Path>) -> Result<()> {
        let mut cmd = Command::new("git");
        if let Some(dir) = dir {
            cmd.current_dir(dir);
        }
        let output = cmd.args(args).output().context("git 実行に失敗しました")?;
        if !output.status.success() {
            bail!(
                "git が失敗しました: {}\nstderr: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn capture_git(args: &[&str], dir: Option<&Path>) -> Result<String> {
        let mut cmd = Command::new("git");
        if let Some(dir) = dir {
            cmd.current_dir(dir);
        }
        let output = cmd.args(args).output().context("git 実行に失敗しました")?;
        if !output.status.success() {
            bail!(
                "git が失敗しました: {}\nstderr: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}
