//! Integration tests for path-style wiki graph helpers.
use std::fs;
use std::path::{Path, PathBuf};

use obsidian_core::Vault;
use zettel::graph::{
    find_backlinks, find_broken_links, load_notes, resolve_wiki_target, source_included,
};

fn write_note(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn path_style_wiki_resolves_and_backlinks() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("folder/target.md"),
        "---\nid: path-target\n---\n# Target\n",
    );
    write_note(
        &dir.path().join("inbox/linker.md"),
        "---\nid: linker\n---\nSee [[folder/target]].\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let target = resolve_wiki_target("folder/target", &notes, vault.path()).unwrap();
    assert_eq!(target.id, "path-target");

    let bl = find_backlinks(&notes, target, vault.path());
    assert_eq!(bl.len(), 1);
    assert!(bl[0].0.path.ends_with("inbox/linker.md"));

    let broken = find_broken_links(&notes, vault.path());
    assert!(
        broken.iter().all(|b| b.target != "folder/target"),
        "path-style link should not be unresolved: {broken:?}"
    );
}

#[test]
fn path_filter_only_affects_reported_sources() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("folder/outside.md"),
        "---\nid: outside-id\n---\n# Out\n",
    );
    write_note(
        &dir.path().join("inbox/src.md"),
        "---\nid: inbox-src\n---\nSee [[outside-id]] and [[really-missing]].\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let broken = find_broken_links(&notes, vault.path());
    let filters = vec![PathBuf::from("inbox")];
    let reported: Vec<_> = broken
        .into_iter()
        .filter(|b| source_included(&b.source_path, vault.path(), &filters, false))
        .collect();
    assert_eq!(reported.len(), 1);
    assert_eq!(reported[0].target, "really-missing");
}

#[test]
fn archive_excluded_from_reports_by_default() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("archive/old.md"),
        "See [[missing-from-archive]].\n",
    );
    write_note(&dir.path().join("notes/ok.md"), "See [[missing-live]].\n");

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let broken = find_broken_links(&notes, vault.path());
    let without_archive: Vec<_> = broken
        .iter()
        .filter(|b| source_included(&b.source_path, vault.path(), &[], false))
        .collect();
    assert_eq!(without_archive.len(), 1);
    assert_eq!(without_archive[0].target, "missing-live");

    let with_archive: Vec<_> = broken
        .iter()
        .filter(|b| source_included(&b.source_path, vault.path(), &[], true))
        .collect();
    assert_eq!(with_archive.len(), 2);
}
