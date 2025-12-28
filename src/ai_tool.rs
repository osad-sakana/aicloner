use std::process::Command;

use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiTool {
    #[default]
    Claude,
    Codex,
}

impl AiTool {
    /// Returns the command name to execute
    pub fn command_name(&self) -> &str {
        match self {
            AiTool::Claude => "claude",
            AiTool::Codex => "codex",
        }
    }

    /// Returns the command name with platform-specific adjustments
    /// On Windows, this adds .cmd extension for npm-installed commands
    pub fn executable_command(&self) -> String {
        let base_name = self.command_name();

        #[cfg(windows)]
        {
            // On Windows, try to use .cmd extension for npm-installed commands
            format!("{}.cmd", base_name)
        }

        #[cfg(not(windows))]
        {
            base_name.to_string()
        }
    }

    /// Returns a human-readable display name
    pub fn display_name(&self) -> &str {
        match self {
            AiTool::Claude => "Claude",
            AiTool::Codex => "Codex",
        }
    }

    /// Checks if the tool is installed and available
    pub fn check_installed(&self) -> Result<()> {
        let command_name = self.command_name();
        let executable = self.executable_command();

        let output = Command::new(&executable)
            .arg("--version")
            .output();

        match output {
            Ok(output) if output.status.success() => Ok(()),
            _ => bail!(
                "{} CLI ({}) がインストールされていません。",
                self.display_name(),
                command_name
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
        assert_eq!(AiTool::Codex.command_name(), "codex");
    }

    #[test]
    fn test_display_names() {
        assert_eq!(AiTool::Claude.display_name(), "Claude");
        assert_eq!(AiTool::Codex.display_name(), "Codex");
    }

    #[test]
    fn test_default_trait() {
        assert_eq!(AiTool::default(), AiTool::Claude);
    }
}
