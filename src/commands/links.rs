use std::path::PathBuf;

use crate::error::{CliError, ExitStatus};
use crate::output::{self, Format};
use crate::vault_util;

pub fn cmd_links(vault: Option<PathBuf>, format: Format, note: &str) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    let resolved = v.resolve_note(note)?;
    let links = &resolved.links;

    match format {
        Format::Json => {
            let out: Vec<_> = links.iter().map(output::link_to_out).collect();
            output::print_json(&out).map_err(|e| CliError::Other(e.to_string()))?;
        }
        Format::Text => output::print_links_text(links),
    }
    Ok(ExitStatus::Ok)
}
