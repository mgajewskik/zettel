use std::io::{self, Write};
use std::path::Path;

use obsidian_core::{Link, LocatedLink, Note};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    #[default]
    Text,
    Json,
}

#[derive(Serialize)]
pub struct LinkOut {
    pub kind: String,
    pub target: String,
    pub heading: Option<String>,
    pub alias: Option<String>,
    pub line: usize,
    pub col_start: usize,
    pub col_end: usize,
    pub resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
}

#[derive(Serialize)]
pub struct BacklinkOut {
    pub path: String,
    pub id: String,
    pub links: Vec<LinkOut>,
}

#[derive(Serialize)]
pub struct BrokenLinkOut {
    pub source_path: String,
    pub line: usize,
    pub text: String,
    /// Parsed wiki target or markdown URL path (without surrounding markup).
    pub target: String,
}

#[derive(Serialize)]
pub struct RuleViolationOut {
    pub source_path: String,
    pub line: usize,
    pub text: String,
    /// Parsed wiki target or markdown URL path (without surrounding markup).
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_path: Option<String>,
}

#[derive(Serialize)]
pub struct NoteOut {
    pub path: String,
    pub id: String,
    pub title: Option<String>,
    pub aliases: Vec<String>,
}

#[derive(Serialize)]
pub struct ExistsOut {
    pub query: String,
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<NoteOut>,
}

pub fn link_to_out(ll: &LocatedLink, resolved_path: Option<&str>) -> LinkOut {
    let resolved = resolved_path.is_some();
    let resolved_path = resolved_path.map(|s| s.to_string());
    match &ll.link {
        Link::Wiki {
            target,
            heading,
            alias,
        } => LinkOut {
            kind: "wiki".into(),
            target: target.clone(),
            heading: heading.clone(),
            alias: alias.clone(),
            line: ll.location.line,
            col_start: ll.location.col_start,
            col_end: ll.location.col_end,
            resolved,
            resolved_path,
        },
        Link::Markdown { text, url } => LinkOut {
            kind: "markdown".into(),
            target: url.clone(),
            heading: None,
            alias: Some(text.clone()),
            line: ll.location.line,
            col_start: ll.location.col_start,
            col_end: ll.location.col_end,
            resolved,
            resolved_path,
        },
        Link::Embed {
            target,
            heading,
            alias,
        } => LinkOut {
            kind: "embed".into(),
            target: target.clone(),
            heading: heading.clone(),
            alias: alias.clone(),
            line: ll.location.line,
            col_start: ll.location.col_start,
            col_end: ll.location.col_end,
            resolved,
            resolved_path,
        },
    }
}

pub fn format_link_text(ll: &LocatedLink) -> String {
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

pub fn note_to_out(note: &Note, vault_root: &Path) -> NoteOut {
    let path = note
        .path
        .strip_prefix(vault_root)
        .unwrap_or(&note.path)
        .display()
        .to_string();
    NoteOut {
        path,
        id: note.id.clone(),
        title: note.title.clone(),
        aliases: note.aliases.clone(),
    }
}

pub fn print_json<T: Serialize>(value: &T) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    writeln!(stdout)?;
    Ok(())
}

pub fn print_links_text(links: &[LocatedLink]) {
    for ll in links {
        println!("{}:{}", ll.location.line, format_link_text(ll));
    }
}

pub fn print_backlinks_text(entries: &[(Note, Vec<LocatedLink>)], vault_root: &Path) {
    for (note, links) in entries {
        let path = note
            .path
            .strip_prefix(vault_root)
            .unwrap_or(&note.path)
            .display();
        println!("{path} ({})", note.id);
        for ll in links {
            println!("  {}:{}", ll.location.line, format_link_text(ll));
        }
    }
}

pub fn print_broken_text(items: &[BrokenLinkOut]) {
    for b in items {
        println!("{}:{}: {}", b.source_path, b.line, b.text);
    }
}

pub fn print_violations_text(items: &[RuleViolationOut]) {
    for b in items {
        match &b.resolved_path {
            Some(rp) => {
                println!("{}:{}: {} → {} (resolved: {})", b.source_path, b.line, b.text, b.target, rp)
            }
            None => println!("{}:{}: {} → {}", b.source_path, b.line, b.text, b.target),
        }
    }
}

pub fn print_notes_text(notes: &[NoteOut]) {
    for n in notes {
        let title = n.title.as_deref().unwrap_or("");
        if title.is_empty() {
            println!("{}\t{}", n.path, n.id);
        } else {
            println!("{}\t{}\t{}", n.path, n.id, title);
        }
    }
}

pub fn rel_path(path: &Path, vault_root: &Path) -> String {
    path.strip_prefix(vault_root)
        .unwrap_or(path)
        .display()
        .to_string()
}
