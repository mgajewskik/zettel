//! Vault link convention rules for `check-links` (`--rule zettel`).

use std::path::{Component, Path, PathBuf};

use obsidian_core::{Link, LocatedLink, Note};

use crate::graph::resolve_link;

/// Reserved top-level folders that are **not** permanent notes under the
/// `zettel` check-links rule (see RULES.md).
pub const NON_PERMANENT_DIRS: &[&str] = &[
    "inbox", "source", "project", "archive", "media", "canvas", "moc",
];

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

/// Whether a note path is a permanent note: vault-relative path is **not**
/// under any of [`NON_PERMANENT_DIRS`].
pub fn is_permanent_note(path: &Path, vault_root: &Path) -> bool {
    !NON_PERMANENT_DIRS
        .iter()
        .any(|d| is_under_dir(path, vault_root, d))
}

/// Whether a vault-relative path (or absolute note path) lies under `source/`.
pub fn is_under_source(path: &Path, vault_root: &Path) -> bool {
    is_under_dir(path, vault_root, "source")
}

/// Whether a wiki/markdown target string indicates a path under `source/`.
pub fn target_indicates_source(target: &str) -> bool {
    let t = norm_slash(target);
    let t = t.strip_prefix("./").unwrap_or(t.as_str());
    let no_md = t.strip_suffix(".md").unwrap_or(t);
    no_md.eq_ignore_ascii_case("source")
        || t.to_ascii_lowercase().starts_with("source/")
        || no_md.to_ascii_lowercase().starts_with("source/")
}

/// A permanent→source link violation for `check-links --rule zettel`.
#[derive(Debug, Clone)]
pub struct RuleViolation {
    pub source_path: PathBuf,
    pub line: usize,
    pub text: String,
    pub target: String,
    pub resolved_path: Option<PathBuf>,
}

fn link_target_string(link: &Link) -> Option<String> {
    match link {
        Link::Wiki { target, .. } | Link::Embed { target, .. } => {
            if target.is_empty() {
                None
            } else {
                Some(target.clone())
            }
        }
        Link::Markdown { url, .. } => local_md_url_path(url).map(|s| s.to_string()),
    }
}

fn link_display_text(ll: &LocatedLink) -> String {
    match &ll.link {
        Link::Wiki {
            target,
            heading,
            alias,
        } => {
            let mut s = format!("[[{target}");
            if let Some(h) = heading {
                s.push('#');
                s.push_str(h);
            }
            if let Some(a) = alias {
                s.push('|');
                s.push_str(a);
            }
            s.push_str("]]");
            s
        }
        Link::Markdown { text, url } => format!("[{text}]({url})"),
        Link::Embed {
            target,
            heading,
            alias,
        } => {
            let mut s = format!("![[{target}");
            if let Some(h) = heading {
                s.push('#');
                s.push_str(h);
            }
            if let Some(a) = alias {
                s.push('|');
                s.push_str(a);
            }
            s.push_str("]]");
            s
        }
    }
}

/// Find permanent notes that link **to** `source/**` (zettel rule).
///
/// Uses [`resolve_link`] (path-style + same-folder basename). A link is a
/// violation when the resolved path is under `source/`, or when the unresolved
/// target string indicates `source/`.
pub fn find_zettel_source_link_violations(notes: &[Note], vault_root: &Path) -> Vec<RuleViolation> {
    let mut out: Vec<RuleViolation> = Vec::new();
    for note in notes {
        if !is_permanent_note(&note.path, vault_root) {
            continue;
        }
        for ll in &note.links {
            let Some(target) = link_target_string(&ll.link) else {
                continue;
            };
            let resolved = resolve_link(note, &ll.link, notes, vault_root);
            let hits_source = match &resolved {
                Some(rp) => is_under_source(rp, vault_root),
                None => target_indicates_source(&target),
            };
            if !hits_source {
                continue;
            }
            out.push(RuleViolation {
                source_path: note.path.clone(),
                line: ll.location.line,
                text: link_display_text(ll),
                target,
                resolved_path: resolved,
            });
        }
    }
    out.sort_by(|a, b| {
        a.source_path
            .cmp(&b.source_path)
            .then(a.line.cmp(&b.line))
            .then(a.target.cmp(&b.target))
    });
    out
}
