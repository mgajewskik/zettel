//! Thin wrappers around `obsidian-rs-core` link resolution that also treat
//! path-style wiki targets (`[[folder/Note]]`) as vault-relative paths, and
//! basename targets with Obsidian-like same-folder then unique-basename rules.
//!
//! Core 0.6 indexes wiki targets by id / stem / alias only. This module does
//! **not** fork the core graph; it layers path and same-folder matching on top
//! for backlinks and broken-link checks.

use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use obsidian_core::{Link, LocatedLink, Note, Vault};

use crate::vault_util;

/// Load notes for graph operations (full vault minus media/canvas).
/// Archive notes are included so links *to* them still resolve.
pub fn load_notes(vault: &Vault) -> Vec<Note> {
    vault
        .notes_filtered(vault_util::default_path_filter)
        .into_iter()
        .filter_map(|r| r.ok())
        .collect()
}

/// Whether a source path should appear in reports / find results.
pub fn source_included(
    path: &Path,
    vault_root: &Path,
    path_filters: &[PathBuf],
    include_archive: bool,
) -> bool {
    if !include_archive && is_under_dir(path, vault_root, "archive") {
        return false;
    }
    if path_filters.is_empty() {
        return true;
    }
    vault_util::path_matches_filters(path, vault_root, path_filters)
}

fn is_under_dir(path: &Path, vault_root: &Path, dir_name: &str) -> bool {
    let rel = path.strip_prefix(vault_root).unwrap_or(path);
    rel.components().any(|c| match c {
        Component::Normal(name) => name.eq_ignore_ascii_case(dir_name),
        _ => false,
    })
}

fn norm_slash(s: &str) -> String {
    s.replace('\\', "/")
}

/// Vault-relative path keys for a note (with and without `.md`).
pub fn note_rel_keys(vault_root: &Path, note: &Note) -> Vec<String> {
    let rel = note.path.strip_prefix(vault_root).unwrap_or(&note.path);
    let with_md = norm_slash(&rel.to_string_lossy());
    let mut keys = vec![with_md.clone()];
    if let Some(stripped) = with_md.strip_suffix(".md") {
        keys.push(stripped.to_string());
    }
    keys
}

fn path_matches_vault_rel(target: &str, note: &Note, vault_root: &Path) -> bool {
    let target = norm_slash(target);
    let target_no_md = target.strip_suffix(".md").unwrap_or(target.as_str());
    for key in note_rel_keys(vault_root, note) {
        let key_no_md = key.strip_suffix(".md").unwrap_or(key.as_str());
        if key == target || key_no_md == target_no_md {
            return true;
        }
    }
    false
}

/// Strip an optional trailing `.md` from a basename-style wiki target.
fn wiki_basename_key(target: &str) -> &str {
    target.strip_suffix(".md").unwrap_or(target)
}

/// Build `dirname(source)/target.md` (target may already include `.md`).
fn source_dir_candidate(source: &Note, target: &str, vault_root: &Path) -> PathBuf {
    let source_dir = source.path.parent().unwrap_or(vault_root);
    let file_name = if target.ends_with(".md") {
        target.to_string()
    } else {
        format!("{target}.md")
    };
    obsidian_core::common::normalize_path(source_dir.join(file_name), Some(vault_root))
}

/// Whether a wiki target matches a note via id / stem / alias **or** vault-relative path.
///
/// Prefer [`resolve_wiki_target`] when a source note is available (same-folder + uniqueness).
pub fn wiki_matches_note(wiki_target: &str, note: &Note, vault_root: &Path) -> bool {
    if wiki_target.is_empty() {
        return false;
    }
    let key = wiki_basename_key(wiki_target);
    if wiki_target == note.id || key == note.id {
        return true;
    }
    if note
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s == wiki_target || s == key)
    {
        return true;
    }
    if note
        .aliases
        .iter()
        .any(|a| a == wiki_target || a.as_str() == key)
    {
        return true;
    }
    if wiki_target.contains('/') {
        return path_matches_vault_rel(wiki_target, note, vault_root);
    }
    false
}

/// Resolve a wiki target with Obsidian-like precedence:
/// 1. Vault-relative path when the target contains `/`
/// 2. Same-folder relative to `source` (`dirname(source)/target.md`)
/// 3. Unique basename match across the vault (optional `.md`); multiple → unresolved
/// 4. Id / alias (stem already covered by basename)
pub fn resolve_wiki_target<'a>(
    target: &str,
    source: &Note,
    notes: &'a [Note],
    vault_root: &Path,
) -> Option<&'a Note> {
    if target.is_empty() {
        return None;
    }
    let target = norm_slash(target);

    // 1. Path-style → vault-relative
    if target.contains('/') {
        return notes
            .iter()
            .find(|n| path_matches_vault_rel(&target, n, vault_root));
    }

    // 2. Same-folder relative to the source note
    let candidate = source_dir_candidate(source, &target, vault_root);
    if let Some(n) = notes.iter().find(|n| n.path == candidate) {
        return Some(n);
    }

    // 3. Unique basename across the vault (treat `b` and `b.md` as the same key)
    let key = wiki_basename_key(&target);
    let basename_hits: Vec<&'a Note> = notes
        .iter()
        .filter(|n| {
            n.path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|stem| stem == key)
        })
        .collect();
    match basename_hits.len() {
        1 => return Some(basename_hits[0]),
        n if n > 1 => return None, // ambiguous — do not silently pick
        _ => {}
    }

    // 4. Id / alias (exact, including optional `.md` stripped form)
    notes.iter().find(|n| {
        n.id == target
            || n.id == key
            || n.aliases
                .iter()
                .any(|a| a == &target || a.as_str() == key)
    })
}

fn local_md_url_path(url: &str) -> Option<&str> {
    if url.contains("://") || url.starts_with('/') {
        return None;
    }
    let url_path = match url.find('#') {
        Some(i) => &url[..i],
        None => url,
    };
    if url_path.ends_with(".md") && !url_path.is_empty() {
        Some(url_path)
    } else {
        None
    }
}

/// Resolve a local markdown URL against `source` and the vault note set.
pub fn resolve_markdown_url<'a>(
    url: &str,
    source: &Note,
    notes: &'a [Note],
    vault_root: &Path,
) -> Option<&'a Note> {
    let url_path = local_md_url_path(url)?;
    let note_paths: HashSet<&Path> = notes.iter().map(|n| n.path.as_path()).collect();
    let source_dir = source.path.parent().unwrap_or(vault_root);
    let candidate =
        obsidian_core::common::normalize_path(source_dir.join(url_path), Some(vault_root));
    if note_paths.contains(candidate.as_path()) {
        return notes.iter().find(|n| n.path == candidate);
    }
    let url_norm = norm_slash(url_path);
    notes.iter().find(|n| {
        note_rel_keys(vault_root, n)
            .iter()
            .any(|k| k == &url_norm)
    })
}

/// Resolve any link kind to an absolute note path when possible.
pub fn resolve_link(
    source: &Note,
    link: &Link,
    notes: &[Note],
    vault_root: &Path,
) -> Option<PathBuf> {
    match link {
        Link::Wiki { target, .. } | Link::Embed { target, .. } => {
            resolve_wiki_target(target, source, notes, vault_root).map(|n| n.path.clone())
        }
        Link::Markdown { url, .. } => {
            resolve_markdown_url(url, source, notes, vault_root).map(|n| n.path.clone())
        }
    }
}

/// Broken link with a parsed target string for JSON consumers.
#[derive(Debug, Clone)]
pub struct BrokenLink {
    pub source_path: PathBuf,
    pub line: usize,
    pub text: String,
    pub target: String,
}

/// Find broken wiki + local markdown links using the **full** note set for resolution.
pub fn find_broken_links(notes: &[Note], vault_root: &Path) -> Vec<BrokenLink> {
    let mut broken: Vec<BrokenLink> = Vec::new();
    for note in notes {
        for ll in &note.links {
            match &ll.link {
                Link::Wiki { target, .. } => {
                    if target.is_empty() {
                        continue;
                    }
                    if resolve_wiki_target(target, note, notes, vault_root).is_none() {
                        broken.push(BrokenLink {
                            source_path: note.path.clone(),
                            line: ll.location.line,
                            text: format!("[[{target}]]"),
                            target: target.clone(),
                        });
                    }
                }
                Link::Markdown { url, .. } => {
                    let Some(url_path) = local_md_url_path(url) else {
                        continue;
                    };
                    if resolve_markdown_url(url, note, notes, vault_root).is_none() {
                        broken.push(BrokenLink {
                            source_path: note.path.clone(),
                            line: ll.location.line,
                            text: format!("[...]({url})"),
                            target: url_path.to_string(),
                        });
                    }
                }
                Link::Embed { .. } => {}
            }
        }
    }
    broken.sort_by(|a, b| {
        a.source_path
            .cmp(&b.source_path)
            .then(a.line.cmp(&b.line))
    });
    broken
}

fn link_targets_note_ext(
    source: &Note,
    link: &Link,
    target: &Note,
    notes: &[Note],
    vault_root: &Path,
) -> bool {
    match link {
        Link::Wiki {
            target: wiki_target, ..
        } => resolve_wiki_target(wiki_target, source, notes, vault_root)
            .is_some_and(|n| n.path == target.path),
        Link::Markdown { url, .. } => {
            let Some(url_path) = local_md_url_path(url) else {
                return false;
            };
            let source_dir = source.path.parent().unwrap_or(&source.path);
            let candidate =
                obsidian_core::common::normalize_path(source_dir.join(url_path), Some(vault_root));
            if candidate == target.path {
                return true;
            }
            let rel = target
                .path
                .strip_prefix(vault_root)
                .unwrap_or(&target.path);
            norm_slash(&rel.to_string_lossy()) == norm_slash(url_path)
        }
        Link::Embed { .. } => false,
    }
}

/// Backlinks including path-style `[[folder/Note]]` and same-folder basename matches.
pub fn find_backlinks(
    notes: &[Note],
    target: &Note,
    vault_root: &Path,
) -> Vec<(Note, Vec<LocatedLink>)> {
    notes
        .iter()
        .filter(|source| source.path != target.path)
        .filter_map(|source| {
            let matching: Vec<LocatedLink> = source
                .links
                .iter()
                .filter(|ll| link_targets_note_ext(source, &ll.link, target, notes, vault_root))
                .cloned()
                .collect();
            if matching.is_empty() {
                None
            } else {
                Some((source.clone(), matching))
            }
        })
        .collect()
}
