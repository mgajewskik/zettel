//! Thin wrappers around `obsidian-rs-core` link resolution that also treat
//! path-style wiki targets (`[[folder/Note]]`) as vault-relative paths.
//!
//! Core 0.6 indexes wiki targets by id / stem / alias only. This module does
//! **not** fork the core graph; it layers path matching on top for backlinks
//! and broken-link checks.

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

/// Whether a wiki target matches a note via id / stem / alias **or** vault-relative path.
pub fn wiki_matches_note(wiki_target: &str, note: &Note, vault_root: &Path) -> bool {
    if wiki_target.is_empty() {
        return false;
    }
    if wiki_target == note.id {
        return true;
    }
    if note
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s == wiki_target)
    {
        return true;
    }
    if note.aliases.iter().any(|a| a == wiki_target) {
        return true;
    }
    if wiki_target.contains('/') {
        let target = norm_slash(wiki_target);
        let target_no_md = target.strip_suffix(".md").unwrap_or(target.as_str());
        for key in note_rel_keys(vault_root, note) {
            let key_no_md = key.strip_suffix(".md").unwrap_or(key.as_str());
            if key == target || key_no_md == target_no_md {
                return true;
            }
        }
    }
    false
}

/// Resolve a wiki target to a note in the full note set.
pub fn resolve_wiki_target<'a>(
    target: &str,
    notes: &'a [Note],
    vault_root: &Path,
) -> Option<&'a Note> {
    notes
        .iter()
        .find(|n| wiki_matches_note(target, n, vault_root))
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
            resolve_wiki_target(target, notes, vault_root).map(|n| n.path.clone())
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
    let mut valid_wiki: HashSet<String> = HashSet::new();
    for note in notes {
        valid_wiki.insert(note.id.clone());
        if let Some(stem) = note.path.file_stem().and_then(|s| s.to_str()) {
            valid_wiki.insert(stem.to_string());
        }
        for alias in &note.aliases {
            valid_wiki.insert(alias.clone());
        }
        for key in note_rel_keys(vault_root, note) {
            valid_wiki.insert(key);
        }
    }

    let mut broken: Vec<BrokenLink> = Vec::new();
    for note in notes {
        for ll in &note.links {
            match &ll.link {
                Link::Wiki { target, .. } => {
                    if target.is_empty() {
                        continue;
                    }
                    let ok = valid_wiki.contains(target.as_str())
                        || (target.contains('/')
                            && resolve_wiki_target(target, notes, vault_root).is_some());
                    if !ok {
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
    vault_root: &Path,
) -> bool {
    match link {
        Link::Wiki {
            target: wiki_target, ..
        } => wiki_matches_note(wiki_target, target, vault_root),
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

/// Backlinks including path-style `[[folder/Note]]` matches.
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
                .filter(|ll| link_targets_note_ext(source, &ll.link, target, vault_root))
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
