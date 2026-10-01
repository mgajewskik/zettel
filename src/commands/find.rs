use std::path::PathBuf;

use clap::ValueEnum;

use crate::error::{CliError, ExitStatus};
use crate::graph;
use crate::output::{self, Format, NoteOut};
use crate::vault_util;

/// Search mode for `find`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum FindMode {
    /// Prefer id / path / title / alias; fall back to content only if none match.
    #[default]
    Auto,
    /// Match note id (case-insensitive contains via exact id search).
    Id,
    /// Match vault-relative path substring.
    Path,
    /// Match title or alias substring.
    Title,
    /// Match note body content.
    Content,
}

/// Search notes. Default `--mode auto` ranks structural hits over content so
/// distill / overlap checks are not drowned by body matches.
pub fn cmd_find(
    vault: Option<PathBuf>,
    format: Format,
    query: &str,
    mode: FindMode,
    include_archive: bool,
) -> Result<ExitStatus, CliError> {
    let v = vault_util::open_vault(vault)?;
    let q_lower = query.to_lowercase();

    let mut notes = match mode {
        FindMode::Id => search_id(&v, query)?,
        FindMode::Path => {
            let all = graph::load_notes(&v);
            all.into_iter()
                .filter(|n| {
                    let rel = output::rel_path(&n.path, v.path()).to_lowercase();
                    rel.contains(&q_lower)
                })
                .collect()
        }
        FindMode::Title => search_title_alias(&v, query)?,
        FindMode::Content => search_content(&v, query)?,
        FindMode::Auto => {
            let mut structural = Vec::new();
            // id
            structural.extend(search_id(&v, query)?);
            // path
            let all = graph::load_notes(&v);
            for n in &all {
                let rel = output::rel_path(&n.path, v.path()).to_lowercase();
                if rel.contains(&q_lower) {
                    structural.push(n.clone());
                }
            }
            // title / alias
            structural.extend(search_title_alias(&v, query)?);
            dedupe_by_path(&mut structural);

            if structural.is_empty() {
                search_content(&v, query)?
            } else {
                // Prefer exact id, then path hits, then title — already roughly ordered;
                // re-rank: exact id first, then path contains, then rest.
                rank_structural(&mut structural, query, v.path());
                structural
            }
        }
    };

    notes.retain(|n| {
        vault_util::default_path_filter(&n.path)
            && graph::source_included(&n.path, v.path(), &[], include_archive)
    });

    let out: Vec<NoteOut> = notes
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

fn search_id(
    v: &obsidian_core::Vault,
    query: &str,
) -> Result<Vec<obsidian_core::Note>, CliError> {
    let results = v
        .search()
        .or_has_id(query)
        .ignore_case()
        .execute()
        .map_err(|e| CliError::Vault(e.to_string()))?;
    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}

fn search_title_alias(
    v: &obsidian_core::Vault,
    query: &str,
) -> Result<Vec<obsidian_core::Note>, CliError> {
    let results = v
        .search()
        .or_title_contains(query)
        .or_alias_contains(query)
        .ignore_case()
        .execute()
        .map_err(|e| CliError::Vault(e.to_string()))?;
    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}

fn search_content(
    v: &obsidian_core::Vault,
    query: &str,
) -> Result<Vec<obsidian_core::Note>, CliError> {
    let results = v
        .search()
        .or_content_contains(query)
        .ignore_case()
        .execute()
        .map_err(|e| CliError::Vault(e.to_string()))?;
    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}

fn dedupe_by_path(notes: &mut Vec<obsidian_core::Note>) {
    let mut seen = std::collections::HashSet::new();
    notes.retain(|n| seen.insert(n.path.clone()));
}

fn rank_structural(notes: &mut [obsidian_core::Note], query: &str, vault_root: &std::path::Path) {
    let q_lower = query.to_lowercase();
    notes.sort_by_key(|n| {
        let exact_id = n.id.eq_ignore_ascii_case(query);
        let path_hit = output::rel_path(&n.path, vault_root)
            .to_lowercase()
            .contains(&q_lower);
        // lower sort key = earlier
        if exact_id {
            0u8
        } else if path_hit {
            1
        } else {
            2
        }
    });
}
