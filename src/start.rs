use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::repo::RepoManager;

pub fn handle_start(issue_number: u32, manager: RepoManager) -> Result<()> {
    // Issue existence verification
    verify_issue_exists(issue_number, &manager)?;

    // Determine base branch
    let base_branch = determine_base_branch(&manager)?;

    // Generate branch name
    let mut branch_name = format!("aicloner/issue{}", issue_number);

    // Check for conflicts and resolve
    if manager.task_exists(&branch_name) {
        branch_name = handle_branch_conflict(&branch_name, issue_number)?;
    }

    // Create workspace
    let workspace_path = create_workspace_for_issue(&manager, &branch_name, &base_branch)?;

    // Launch Claude session
    launch_claude_session(&workspace_path, issue_number)?;

    Ok(())
}

fn verify_issue_exists(issue_number: u32, manager: &RepoManager) -> Result<()> {
    let base_dir = manager.base_dir();

    let args = vec![
        "issue".to_string(),
        "view".to_string(),
        issue_number.to_string(),
    ];

    println!("実行: gh {} (cwd: {})", args.join(" "), base_dir.display());
    let output = Command::new("gh")
        .args(&args)
        .current_dir(&base_dir)
        .output()
        .context("gh issue view の実行に失敗しました")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Issue #{} が見つかりません: {}", issue_number, stderr.trim());
    }

    println!("✓ Issue #{} を確認しました", issue_number);
    Ok(())
}

fn determine_base_branch(manager: &RepoManager) -> Result<String> {
    let base_dir = manager.base_dir();

    let args = vec![
        "-C".to_string(),
        base_dir.display().to_string(),
        "rev-parse".to_string(),
        "--abbrev-ref".to_string(),
        "HEAD".to_string(),
    ];

    println!("実行: git {}", args.join(" "));
    let output = Command::new("git")
        .args(&args)
        .output()
        .context("base ディレクトリのブランチ取得に失敗しました")?;

    if !output.status.success() {
        bail!("base ディレクトリのブランチを取得できませんでした");
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    println!("✓ ベースブランチ: {}", branch);
    Ok(branch)
}

fn handle_branch_conflict(branch_name: &str, issue_number: u32) -> Result<String> {
    println!("ブランチ \"{}\" は既に存在します。", branch_name);
    println!("選択してください:");
    println!("  1. 既存のワークスペースに切り替える");
    println!("  2. 新しいブランチ名で作成する (例: aicloner/issue{}-2)", issue_number);
    println!("  3. キャンセル");

    print!("選択 [1-3]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => Ok(branch_name.to_string()),
        "2" => {
            print!("新しいブランチ名を入力してください: ");
            io::stdout().flush()?;
            let mut new_branch = String::new();
            io::stdin().read_line(&mut new_branch)?;
            let new_branch = new_branch.trim().to_string();
            if new_branch.is_empty() {
                bail!("ブランチ名が空です");
            }
            Ok(new_branch)
        }
        "3" => bail!("操作をキャンセルしました"),
        _ => bail!("操作をキャンセルしました"),
    }
}

fn create_workspace_for_issue(
    manager: &RepoManager,
    branch_name: &str,
    base_branch: &str,
) -> Result<PathBuf> {
    manager.create_task_clone(branch_name, base_branch)?;
    Ok(manager.workspaces_dir().join(branch_name))
}

fn launch_claude_session(workspace_path: &Path, issue_number: u32) -> Result<()> {
    let prompt = format!(
        "あなたは優秀なエンジニアです。issue#{}を対応してください。\n\n\
         - ghコマンドを使ってissueを確認すること\n\
         - issueに従って適切に実装すること\n\
         - 疑問点はユーザーに聞くこと\n\
         - ghコマンドを使って実装後にプルリクエストにすること",
        issue_number
    );

    println!("\nClaudeセッションを起動します...");
    println!("ワークスペース: {}", workspace_path.display());

    // Change to workspace directory and exec claude
    std::env::set_current_dir(workspace_path)
        .with_context(|| format!("ワークスペースへの移動に失敗しました: {}", workspace_path.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = Command::new("claude").arg(&prompt).exec();
        // exec only returns on error
        Err(anyhow::anyhow!("Claude の起動に失敗しました: {}", err))
    }

    #[cfg(not(unix))]
    {
        // For Windows: use spawn and wait
        let status = Command::new("claude").arg(&prompt).status()?;

        if !status.success() {
            bail!("Claude が異常終了しました");
        }
        Ok(())
    }
}
