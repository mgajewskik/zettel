use std::fmt;
use std::process::ExitCode;

/// Process exit codes for the zettel CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitStatus {
    Ok = 0,
    /// Command completed but found issues (e.g. unresolved links, note missing).
    Issues = 1,
    /// Usage or configuration error (missing vault, bad args).
    Usage = 2,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        ExitCode::from(status as u8)
    }
}

#[derive(Debug)]
pub enum CliError {
    Config(String),
    Vault(String),
    NoteNotFound(String),
    Ambiguous(String),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Config(m) => write!(f, "{m}"),
            CliError::Vault(m) => write!(f, "{m}"),
            CliError::NoteNotFound(m) => write!(f, "note not found: {m}"),
            CliError::Ambiguous(m) => write!(f, "ambiguous note identifier: {m}"),
            CliError::Other(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for CliError {}

impl CliError {
    pub fn exit_status(&self) -> ExitStatus {
        match self {
            CliError::Config(_) => ExitStatus::Usage,
            CliError::NoteNotFound(_) => ExitStatus::Issues,
            _ => ExitStatus::Usage,
        }
    }
}

impl From<obsidian_core::VaultError> for CliError {
    fn from(err: obsidian_core::VaultError) -> Self {
        match err {
            obsidian_core::VaultError::NoteNotFound(s) => CliError::NoteNotFound(s),
            obsidian_core::VaultError::AmbiguousNoteIdentifier(s, paths) => {
                let joined = paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                CliError::Ambiguous(format!("{s} → [{joined}]"))
            }
            other => CliError::Vault(other.to_string()),
        }
    }
}
