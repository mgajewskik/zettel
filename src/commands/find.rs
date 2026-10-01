use std::path::PathBuf;

use crate::error::{CliError, ExitStatus};
use crate::output::{self, Format};
use crate::vault_util;

/// Search notes by id, title, alias, or content substring (case-insensitive).
pub fn cmd_find(
    vault: Option<PathBuf>,
    format: Format,
    query: &str,
) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;

    let results = v
        .search()
        .or_has_id(query)
        .or_title_contains(query)
        .or_alias_contains(query)
        .or_content_contains(query)
        .ignore_case()
        .execute()
        .map_err(|e| CliError::Vault(e.to_string()))?;

    let notes: Vec<_> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .filter(|n| vault_util::default_path_filter(&n.path))
        .collect();

    let out: Vec<_> = notes
        .iter()
        .map(|n| output::note_to_out(n, v.path()))
        .collect();

    match format {
        Format::Json => {
            output::print_json(&out).map_err(|e| CliError::Other(e.to_string()))?;
        }
        Format::Text => {
            if out.is_empty() {
                eprintln!("no matches for: {query}");
            } else {
                output::print_notes_text(&out);
            }
        }
    }

    if out.is_empty() {
        Ok(ExitStatus::Issues)
    } else {
        Ok(ExitStatus::Ok)
    }
}
