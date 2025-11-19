use std::{fs, path::Path};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repo_url: String,
    pub base_dir: String,
    pub workspaces_dir: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        if !path.exists() {
            bail!("設定ファイルが見つかりません: {}", path.display());
        }
        // TOML ファイルを文字列として読み込む
        let raw = fs::read_to_string(path)
            .with_context(|| format!("設定ファイルの読み込みに失敗しました: {}", path.display()))?;
        let config = toml::from_str(&raw).context("設定ファイルのパースに失敗しました")?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                // 親ディレクトリが無ければ作成
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "設定ファイルディレクトリの作成に失敗しました: {}",
                        parent.display()
                    )
                })?;
            }
        }
        let content =
            toml::to_string_pretty(self).context("設定ファイルのシリアライズに失敗しました")?;
        fs::write(path, content)
            .with_context(|| format!("設定ファイルの書き込みに失敗しました: {}", path.display()))?;
        Ok(())
    }
}
