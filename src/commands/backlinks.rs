use std::path::PathBuf;

use crate::error::{CliError, ExitStatus};
use crate::graph;
use crate::output::{self, Format};
use crate::vault_util;

pub fn cmd_backlinks(
    vault: Option<PathBuf>,
    format: Format,
    note: &str,
) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    let notes = graph::load_notes(&v);
    let resolved = v.resolve_note(note)?;
    let entries = graph::find_backlinks(&notes, &resolved, v.path());

    match format {
        Format::Json => {
            let out: Vec<_> = entries
                .iter()
                .map(|(n, links)| output::BacklinkOut {
                    path: output::rel_path(&n.path, v.path()),
                    id: n.id.clone(),
                    links: links
                        .iter()
                        .map(|ll| {
                            let rp = graph::resolve_link(n, &ll.link, &notes, v.path())
                                .map(|p| output::rel_path(&p, v.path()));
                            output::link_to_out(ll, rp.as_deref())
                        })
                        .collect(),
                })
                .collect();
            output::print_json(&out).map_err(|e| CliError::Other(e.to_string()))?;
        }
        Format::Text => output::print_backlinks_text(&entries, v.path()),
    }
    Ok(ExitStatus::Ok)
}
