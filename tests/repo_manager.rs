use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use aicloner::{config::Config, repo::RepoManager};
use anyhow::{bail, Context, Result};
use tempfile::TempDir;

#[test]
fn create_clone_and_list_returns_branch() -> Result<()> {
    let tmp = TempDir::new()?;
    let remote = init_remote_repo(&tmp)?;

    let config_path = tmp.path().join(".aicloner.toml");
    let config = Config {
        repo_url: remote.to_string_lossy().to_string(),
        workspaces_dir: "ws".to_string(),
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
        workspaces_dir: "ws".to_string(),
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
