use std::path::PathBuf;

use crate::error::{CliError, ExitStatus};
use crate::output::{self, ExistsOut, Format};
use crate::vault_util;

/// Resolve a note by id/alias/path/title. Exit 0 if found, 1 if not.
pub fn cmd_exists(
    vault: Option<PathBuf>,
    format: Format,
    query: &str,
) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    match v.resolve_note(query) {
        Ok(note) => {
            let out = ExistsOut {
                query: query.to_string(),
                exists: true,
                note: Some(output::note_to_out(&note, v.path())),
            };
            match format {
                Format::Json => {
                    output::print_json(&out).map_err(|e| CliError::Other(e.to_string()))?;
                }
                Format::Text => {
                    let n = out.note.as_ref().unwrap();
                    println!("{}\t{}", n.path, n.id);
                }
            }
            Ok(ExitStatus::Ok)
        }
        Err(obsidian_core::VaultError::NoteNotFound(_)) => {
            let out = ExistsOut {
                query: query.to_string(),
                exists: false,
                note: None,
            };
            match format {
                Format::Json => {
                    output::print_json(&out).map_err(|e| CliError::Other(e.to_string()))?;
                }
                Format::Text => {
                    eprintln!("not found: {query}");
                }
            }
            Ok(ExitStatus::Issues)
        }
        Err(other) => Err(other.into()),
    }
}
