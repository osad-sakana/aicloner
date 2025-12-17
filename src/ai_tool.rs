use std::process::Command;

use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiTool {
    #[default]
    Claude,
    Codex,
    Gemini,
}

impl AiTool {
    /// Returns the command name to execute
    pub fn command_name(&self) -> &str {
        match self {
            AiTool::Claude => "claude",
            AiTool::Codex => "codex",
            AiTool::Gemini => "gemini",
        }
    }

    /// Returns a human-readable display name
    pub fn display_name(&self) -> &str {
        match self {
            AiTool::Claude => "Claude",
            AiTool::Codex => "Codex",
            AiTool::Gemini => "Gemini",
        }
    }

    /// Checks if the tool is installed and available
    pub fn check_installed(&self) -> Result<()> {
        let output = Command::new(self.command_name())
            .arg("--version")
            .output();

        match output {
            Ok(output) if output.status.success() => Ok(()),
            _ => bail!(
                "{} CLI ({}) がインストールされていません。",
                self.display_name(),
                self.command_name()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_names() {
        assert_eq!(AiTool::Claude.command_name(), "claude");
        assert_eq!(AiTool::Codex.command_name(), "codex-cli");
        assert_eq!(AiTool::Gemini.command_name(), "gemini-cli");
    }

    #[test]
    fn test_display_names() {
        assert_eq!(AiTool::Claude.display_name(), "Claude");
        assert_eq!(AiTool::Codex.display_name(), "Codex");
        assert_eq!(AiTool::Gemini.display_name(), "Gemini");
    }

    #[test]
    fn test_default_trait() {
        assert_eq!(AiTool::default(), AiTool::Claude);
    }
}
