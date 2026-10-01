use std::path::PathBuf;

use crate::error::{CliError, ExitStatus};
use crate::output::{self, BrokenLinkOut, Format};
use crate::vault_util;

/// List broken (unresolved) links. Optional path filters restrict which source notes are scanned.
pub fn cmd_unresolved(
    vault: Option<PathBuf>,
    format: Format,
    path_filters: &[PathBuf],
) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    let filter = vault_util::make_filter(v.path(), path_filters);
    let report = v.check(filter);

    let items: Vec<BrokenLinkOut> = report
        .broken_links
        .iter()
        .map(|b| BrokenLinkOut {
            source_path: output::rel_path(&b.source_path, v.path()),
            line: b.line,
            text: b.text.clone(),
        })
        .collect();

    match format {
        Format::Json => {
            output::print_json(&items).map_err(|e| CliError::Other(e.to_string()))?;
        }
        Format::Text => {
            if items.is_empty() {
                eprintln!("no unresolved links");
            } else {
                output::print_broken_text(&items);
            }
        }
    }

    if items.is_empty() {
        Ok(ExitStatus::Ok)
    } else {
        Ok(ExitStatus::Issues)
    }
}
