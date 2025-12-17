use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use anyhow::Result;

use aicloner::config::Config;
use aicloner::repo::RepoManager;

fn init_remote_repo(tmp: &TempDir) -> Result<PathBuf> {
    let remote_path = tmp.path().join("remote.git");
    fs::create_dir(&remote_path)?;

    let init_args = vec!["init".to_string(), "--bare".to_string()];
    std::process::Command::new("git")
        .args(&init_args)
        .current_dir(&remote_path)
        .output()?;

    let dummy_path = tmp.path().join("dummy");
    fs::create_dir(&dummy_path)?;
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&dummy_path)
        .output()?;

    fs::write(dummy_path.join("README.md"), "# Test")?;
    std::process::Command::new("git")
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com", "add", "."])
        .current_dir(&dummy_path)
        .output()?;
    std::process::Command::new("git")
        .args(["-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-m", "init"])
        .current_dir(&dummy_path)
        .output()?;
    std::process::Command::new("git")
        .args(["remote", "add", "origin", remote_path.to_str().unwrap()])
        .current_dir(&dummy_path)
        .output()?;
    std::process::Command::new("git")
        .args(["push", "-u", "origin", "main"])
        .current_dir(&dummy_path)
        .output()?;

    Ok(remote_path)
}

#[test]
fn task_exists_returns_true_for_existing_workspace() -> Result<()> {
    let tmp = TempDir::new()?;
    let remote = init_remote_repo(&tmp)?;

    let config_path = tmp.path().join(".aicloner.toml");
    let config = Config {
        repo_url: remote.to_string_lossy().to_string(),
        base_dir: "base".to_string(),
        workspaces_dir: "ws".to_string(),
    };
    config.save(&config_path)?;

    let manager = RepoManager::new(config, config_path);
    manager.init_environment("main")?;

    // Create a task workspace
    manager.create_task_clone("test-task", "main")?;

    // Check task_exists
    assert!(manager.task_exists("test-task"));
    assert!(!manager.task_exists("nonexistent"));

    Ok(())
}

#[test]
fn task_exists_returns_false_when_workspace_not_created() -> Result<()> {
    let tmp = TempDir::new()?;
    let remote = init_remote_repo(&tmp)?;

    let config_path = tmp.path().join(".aicloner.toml");
    let config = Config {
        repo_url: remote.to_string_lossy().to_string(),
        base_dir: "base".to_string(),
        workspaces_dir: "ws".to_string(),
    };
    config.save(&config_path)?;

    let manager = RepoManager::new(config, config_path);
    manager.init_environment("main")?;

    // Check task_exists before creating any tasks
    assert!(!manager.task_exists("test-task"));

    Ok(())
}
