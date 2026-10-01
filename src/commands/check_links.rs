use std::path::PathBuf;

use clap::ValueEnum;

use crate::error::{CliError, ExitStatus};
use crate::graph;
use crate::output::{self, Format, RuleViolationOut};
use crate::vault_util;

/// Link-check rule set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum CheckRule {
    /// Permanent notes must not link to `source/**`.
    Zettel,
}

/// Enforce vault link conventions (`--rule zettel`).
///
/// Path filters restrict which *permanent source* notes are checked.
/// Resolution always uses the full vault graph (minus media/canvas).
/// Archive permanents are excluded by default; pass `include_archive` to opt in.
pub fn cmd_check_links(
    vault: Option<PathBuf>,
    format: Format,
    rule: CheckRule,
    path_filters: &[PathBuf],
    include_archive: bool,
) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    let notes = graph::load_notes(&v);

    let violations = match rule {
        CheckRule::Zettel => graph::find_zettel_source_link_violations(&notes, v.path()),
    };

    let items: Vec<RuleViolationOut> = violations
        .into_iter()
        .filter(|b| {
            graph::source_included(&b.source_path, v.path(), path_filters, include_archive)
        })
        .map(|b| RuleViolationOut {
            source_path: output::rel_path(&b.source_path, v.path()),
            line: b.line,
            text: b.text,
            target: b.target,
            resolved_path: b
                .resolved_path
                .as_ref()
                .map(|p| output::rel_path(p, v.path())),
        })
        .collect();

    match format {
        Format::Json => {
            output::print_json(&items).map_err(|e| CliError::Other(e.to_string()))?;
        }
        Format::Text => {
            if items.is_empty() {
                eprintln!("no check-links violations");
            } else {
                output::print_violations_text(&items);
            }
        }
    }

    if items.is_empty() {
        Ok(ExitStatus::Ok)
    } else {
        Ok(ExitStatus::Issues)
    }
}
