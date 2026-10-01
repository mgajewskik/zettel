use std::path::PathBuf;

use crate::error::{CliError, ExitStatus};
use crate::graph;
use crate::output::{self, Format};
use crate::vault_util;

pub fn cmd_links(vault: Option<PathBuf>, format: Format, note: &str) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    let notes = graph::load_notes(&v);
    let resolved = v.resolve_note(note)?;
    let links = &resolved.links;

    match format {
        Format::Json => {
            let out: Vec<_> = links
                .iter()
                .map(|ll| {
                    let rp = graph::resolve_link(&resolved, &ll.link, &notes, v.path())
                        .map(|p| output::rel_path(&p, v.path()));
                    output::link_to_out(ll, rp.as_deref())
                })
                .collect();
            output::print_json(&out).map_err(|e| CliError::Other(e.to_string()))?;
        }
        Format::Text => output::print_links_text(links),
    }
    Ok(ExitStatus::Ok)
}
