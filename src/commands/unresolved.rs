use std::path::PathBuf;

use crate::error::{CliError, ExitStatus};
use crate::graph;
use crate::output::{self, BrokenLinkOut, Format};
use crate::vault_util;

/// List broken (unresolved) links.
///
/// Path filters restrict which *source* notes are reported. Resolution always
/// uses the full vault graph (minus media/canvas). Archive sources are excluded
/// by default; pass `include_archive` to opt in.
pub fn cmd_unresolved(
    vault: Option<PathBuf>,
    format: Format,
    path_filters: &[PathBuf],
    include_archive: bool,
) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    let notes = graph::load_notes(&v);
    let broken = graph::find_broken_links(&notes, v.path());

    let items: Vec<BrokenLinkOut> = broken
        .into_iter()
        .filter(|b| {
            graph::source_included(&b.source_path, v.path(), path_filters, include_archive)
        })
        .map(|b| BrokenLinkOut {
            source_path: output::rel_path(&b.source_path, v.path()),
            line: b.line,
            text: b.text,
            target: b.target,
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
