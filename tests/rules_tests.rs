//! Unit/integration tests for check-links --rule zettel.
use std::fs;
use std::path::Path;

use obsidian_core::Vault;
use zettel::graph::load_notes;
use zettel::rules::{find_zettel_source_link_violations, is_permanent_note};

fn write_note(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn zettel_rule_permanent_to_source_flagged() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("idea.md"),
        "---\nid: idea\n---\nSee [[source/ref]].\n",
    );
    write_note(
        &dir.path().join("source/ref.md"),
        "---\nid: source-ref\n---\n# Ref\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let v = find_zettel_source_link_violations(&notes, vault.path());
    assert_eq!(v.len(), 1);
    assert!(v[0].source_path.ends_with("idea.md"));
    assert_eq!(v[0].target, "source/ref");
    assert!(
        v[0].resolved_path
            .as_ref()
            .is_some_and(|p| p.ends_with("source/ref.md"))
    );
}

#[test]
fn zettel_rule_source_to_permanent_ok() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("idea.md"),
        "---\nid: idea\n---\n# Idea\n",
    );
    write_note(
        &dir.path().join("source/ref.md"),
        "---\nid: source-ref\n---\nBack to [[idea]].\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let v = find_zettel_source_link_violations(&notes, vault.path());
    assert!(v.is_empty(), "source→permanent must not be flagged: {v:?}");
}

#[test]
fn zettel_rule_permanent_to_permanent_ok() {
    let dir = tempfile::tempdir().unwrap();
    write_note(&dir.path().join("a.md"), "---\nid: a\n---\nSee [[b]].\n");
    write_note(&dir.path().join("b.md"), "---\nid: b\n---\n# B\n");

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let v = find_zettel_source_link_violations(&notes, vault.path());
    assert!(v.is_empty(), "permanent→permanent must not be flagged: {v:?}");
}

#[test]
fn zettel_rule_area_to_source_not_flagged() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("area/topic.md"),
        "---\nid: area-topic\n---\nSee [[source/ref]].\n",
    );
    write_note(
        &dir.path().join("source/ref.md"),
        "---\nid: source-ref\n---\n# Ref\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let v = find_zettel_source_link_violations(&notes, vault.path());
    assert!(
        v.is_empty(),
        "area/** is not a permanent: must not be flagged: {v:?}"
    );
}

#[test]
fn zettel_rule_root_moc_to_source_flagged() {
    let dir = tempfile::tempdir().unwrap();
    write_note(
        &dir.path().join("moc-overview.md"),
        "---\nid: moc-overview\n---\nSee [[source/ref]].\n",
    );
    write_note(
        &dir.path().join("source/ref.md"),
        "---\nid: source-ref\n---\n# Ref\n",
    );

    let vault = Vault::open(dir.path()).unwrap();
    let notes = load_notes(&vault);
    let v = find_zettel_source_link_violations(&notes, vault.path());
    assert_eq!(v.len(), 1);
    assert!(v[0].source_path.ends_with("moc-overview.md"));
}

#[test]
fn is_permanent_only_vault_root_md() {
    let root = Path::new("/vault");
    assert!(is_permanent_note(Path::new("/vault/idea.md"), root));
    assert!(is_permanent_note(Path::new("/vault/moc-overview.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/notes/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/area/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/inbox/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/source/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/project/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/archive/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/moc/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/docs/x.md"), root));
    assert!(!is_permanent_note(Path::new("/vault/idea.txt"), root));
}
