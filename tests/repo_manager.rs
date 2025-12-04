use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use aicloner::{config::Config, repo::RepoManager};
use anyhow::{bail, Context, Result};
use tempfile::TempDir;

fn git_command(dir: Option<&Path>) -> Command {
    let mut cmd = Command::new("git");
    if let Some(dir) = dir {
        cmd.current_dir(dir);
    }
    // Avoid user/system git config & attributes to keep tests deterministic
    let null_path = if cfg!(windows) { "NUL" } else { "/dev/null" };
    cmd.env("GIT_CONFIG_GLOBAL", null_path)
        .env("GIT_CONFIG_SYSTEM", null_path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_ATTR_NOSYSTEM", "1");
    cmd
}

#[test]
fn create_clone_and_list_returns_branch() -> Result<()> {
    let tmp = TempDir::new()?;
    let remote = init_remote_repo(&tmp)?;
    create_remote_branch_with_commit(&remote, "task-a", "feature")?;

    let config_path = tmp.path().join(".aicloner.toml");
    let config = Config {
        repo_url: remote.to_string_lossy().to_string(),
        base_dir: "base".to_string(),
        workspaces_dir: "ws".to_string(),
    };
    let manager = RepoManager::new(config, config_path);
    manager.init_environment()?;
    assert!(manager.base_dir().exists());

    manager.create_task_clone("task-a", "main")?;

    let workspace = manager.workspaces_dir().join("task-a");
    assert!(workspace.exists());
    let branch = current_branch(&workspace)?;
    assert_eq!(branch, "task-a");
    let content = fs::read_to_string(workspace.join("README.md"))?;
    assert_eq!(normalize_newlines(&content), "feature\n");

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
        base_dir: "base".to_string(),
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

#[test]
fn create_new_branch_when_remote_missing() -> Result<()> {
    let tmp = TempDir::new()?;
    let remote = init_remote_repo(&tmp)?;

    let config_path = tmp.path().join(".aicloner.toml");
    let config = Config {
        repo_url: remote.to_string_lossy().to_string(),
        base_dir: "base".to_string(),
        workspaces_dir: "ws".to_string(),
    };
    let manager = RepoManager::new(config, config_path);
    manager.init_environment()?;

    manager.create_task_clone("task-new", "main")?;

    let workspace = manager.workspaces_dir().join("task-new");
    assert!(workspace.exists());
    let branch = current_branch(&workspace)?;
    assert_eq!(branch, "task-new");
    let content = fs::read_to_string(workspace.join("README.md"))?;
    assert_eq!(normalize_newlines(&content), "hello\n");
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

fn create_remote_branch_with_commit(remote: &Path, branch: &str, content: &str) -> Result<()> {
    let tmp = TempDir::new()?;
    let work = tmp.path().join("work");
    run_git(
        &[
            "clone",
            remote.to_string_lossy().as_ref(),
            work.to_string_lossy().as_ref(),
        ],
        None,
    )?;
    run_git(&["config", "user.name", "tester"], Some(&work))?;
    run_git(&["config", "user.email", "tester@example.com"], Some(&work))?;
    run_git(&["checkout", "-b", branch], Some(&work))?;
    let mut file = fs::File::create(work.join("README.md"))?;
    writeln!(file, "{content}")?;
    run_git(&["add", "README.md"], Some(&work))?;
    run_git(&["commit", "-m", "feat"], Some(&work))?;
    run_git(&["push", "origin", branch], Some(&work))?;
    Ok(())
}

fn current_branch(repo: &Path) -> Result<String> {
    capture_git(&["rev-parse", "--abbrev-ref", "HEAD"], Some(repo))
}

fn run_git(args: &[&str], dir: Option<&Path>) -> Result<()> {
    let output = git_command(dir)
        .args(args)
        .output()
        .context("git 実行に失敗しました")?;
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
    let output = git_command(dir)
        .args(args)
        .output()
        .context("git 実行に失敗しました")?;
    if !output.status.success() {
        bail!(
            "git が失敗しました: {}\nstderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}
