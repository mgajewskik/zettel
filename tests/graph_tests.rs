//! Integration tests for path-style and same-folder wiki graph helpers.
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
    let linker = notes
        .iter()
        .find(|n| n.path.ends_with("inbox/linker.md"))
        .unwrap();
    let target = resolve_wiki_target("folder/target", linker, &notes, vault.path()).unwrap();
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
fn same_folder_basename_resolves_with_and_without_md() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("inbox/b.md"),
        "---\nid: note-b\n---\n# B\n",
    );
    write_note(
        &dir.path().join("inbox/a.md"),
        "---\nid: note-a\n---\nSee [[b]] and [[b.md]].\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let a = notes
        .iter()
        .find(|n| n.path.ends_with("inbox/a.md"))
        .unwrap();
    let b = resolve_wiki_target("b", a, &notes, vault.path()).unwrap();
    assert!(b.path.ends_with("inbox/b.md"));
    let b_md = resolve_wiki_target("b.md", a, &notes, vault.path()).unwrap();
    assert_eq!(b.path, b_md.path);

    let bl = find_backlinks(&notes, b, vault.path());
    assert_eq!(bl.len(), 1);
    assert!(bl[0].0.path.ends_with("inbox/a.md"));

    let broken = find_broken_links(&notes, vault.path());
    assert!(
        broken
            .iter()
            .all(|x| x.target != "b" && x.target != "b.md"),
        "same-folder basename should resolve: {broken:?}"
    );
}

#[test]
fn same_folder_wins_over_other_folder_duplicate_basename() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("inbox/b.md"),
        "---\nid: inbox-b\n---\n# Inbox B\n",
    );
    write_note(
        &dir.path().join("other/b.md"),
        "---\nid: other-b\n---\n# Other B\n",
    );
    write_note(
        &dir.path().join("inbox/a.md"),
        "---\nid: note-a\n---\nSee [[b]].\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let a = notes
        .iter()
        .find(|n| n.path.ends_with("inbox/a.md"))
        .unwrap();
    let resolved = resolve_wiki_target("b", a, &notes, vault.path()).unwrap();
    assert!(
        resolved.path.ends_with("inbox/b.md"),
        "same-folder note must win: {:?}",
        resolved.path
    );
    assert_eq!(resolved.id, "inbox-b");
}

#[test]
fn ambiguous_basename_without_same_folder_stays_unresolved() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("folder1/b.md"),
        "---\nid: b-one\n---\n# B1\n",
    );
    write_note(
        &dir.path().join("folder2/b.md"),
        "---\nid: b-two\n---\n# B2\n",
    );
    write_note(
        &dir.path().join("inbox/linker.md"),
        "---\nid: linker\n---\nSee [[b]].\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let linker = notes
        .iter()
        .find(|n| n.path.ends_with("inbox/linker.md"))
        .unwrap();
    assert!(
        resolve_wiki_target("b", linker, &notes, vault.path()).is_none(),
        "ambiguous basename must not silently resolve"
    );

    let broken = find_broken_links(&notes, vault.path());
    assert!(
        broken.iter().any(|b| b.target == "b"),
        "ambiguous basename should be unresolved: {broken:?}"
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
