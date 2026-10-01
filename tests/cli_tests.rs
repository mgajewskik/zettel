use std::path::PathBuf;
use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;

fn fixture_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vault")
}

fn zettel() -> Command {
    let mut cmd = Command::cargo_bin("zettel").unwrap();
    cmd.arg("--vault").arg(fixture_vault());
    cmd
}

#[test]
fn missing_vault_exits_2() {
    let mut cmd = Command::cargo_bin("zettel").unwrap();
    cmd.env_remove("ZETTEL_VAULT");
    cmd.args(["links", "alpha"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("ZETTEL_VAULT"));
}

#[test]
fn links_outbound() {
    zettel()
        .args(["links", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[[beta]]"))
        .stdout(predicate::str::contains("[[missing-note]]"));
}

#[test]
fn links_json_includes_resolved_fields() {
    zettel()
        .args(["--format", "json", "links", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\""))
        .stdout(predicate::str::contains("\"resolved\""))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn backlinks_to_alpha() {
    zettel()
        .args(["backlinks", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn path_style_backlinks_include_linker() {
    // note A (inbox/linker) links [[folder/target]]; target exists → backlinks include A
    zettel()
        .args(["backlinks", "folder/target"])
        .assert()
        .success()
        .stdout(predicate::str::contains("inbox/linker"));
}

#[test]
fn path_style_unresolved_not_flagged() {
    // path-style links that resolve must not appear in unresolved
    let assert = zettel().args(["--format", "json", "unresolved"]).assert();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("folder/target"),
        "resolved path-style wiki should not be unresolved: {stdout}"
    );
    assert!(
        !stdout.contains("project/x/note"),
        "resolved path-style wiki should not be unresolved: {stdout}"
    );
}

#[test]
fn unresolved_reports_missing_and_exit_1() {
    zettel()
        .args(["unresolved"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("missing-note"));
}

#[test]
fn unresolved_json_has_target_field() {
    zettel()
        .args(["--format", "json", "unresolved"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"target\""))
        .stdout(predicate::str::contains("\"text\""));
}

#[test]
fn unresolved_path_filter_sources_only_exit_1() {
    // Filter restricts *sources* only; resolution still uses full vault.
    // inbox/linker has [[folder/target]] + [[project/x/note]] (both exist) + missing-inbox-only.
    zettel()
        .args(["unresolved", "inbox"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("missing-inbox-only"))
        .stdout(predicate::str::contains("inbox/"))
        .stdout(predicate::str::contains("missing-inbox-only").and(
            predicate::function(|s: &str| {
                !s.contains("folder/target") && !s.contains("project/x/note")
            }),
        ));
}

#[test]
fn unresolved_excludes_archive_by_default() {
    let assert = zettel().args(["unresolved"]).assert().failure().code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains("missing-from-archive"),
        "archive sources excluded by default: {stdout}"
    );
}

#[test]
fn unresolved_include_archive() {
    zettel()
        .args(["unresolved", "--include-archive"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("missing-from-archive"));
}

#[test]
fn unresolved_empty_after_filter_exit_0() {
    // notes/gamma has no broken links; filtering to a path with none → exit 0
    // Use a path that has no broken sources. folder/ only has target.md with no links.
    zettel()
        .args(["unresolved", "folder"])
        .assert()
        .success()
        .code(0);
}

#[test]
fn exists_true_and_false() {
    zettel()
        .args(["exists", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"));

    zettel()
        .args(["exists", "no-such-note-xyz"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn exists_by_alias() {
    zettel()
        .args(["exists", "Alpha Alias"])
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"));
}

#[test]
fn exists_by_path_style() {
    zettel()
        .args(["exists", "folder/target"])
        .assert()
        .success()
        .stdout(predicate::str::contains("folder/target"));
}

#[test]
fn find_matches() {
    zettel()
        .args(["find", "gamma"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gamma"));
}

#[test]
fn find_mode_path() {
    zettel()
        .args(["find", "--mode", "path", "folder/target"])
        .assert()
        .success()
        .stdout(predicate::str::contains("folder/target"));
}

#[test]
fn find_mode_id() {
    zettel()
        .args(["find", "--mode", "id", "path-target"])
        .assert()
        .success()
        .stdout(predicate::str::contains("folder/target"));
}

#[test]
fn find_excludes_archive_by_default() {
    zettel()
        .args(["find", "--mode", "id", "archived-note"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn find_include_archive() {
    zettel()
        .args(["find", "--mode", "id", "--include-archive", "archived-note"])
        .assert()
        .success()
        .stdout(predicate::str::contains("archive/old"));
}

#[test]
fn find_miss_exit_1() {
    zettel()
        .args(["find", "zzzz-no-match-zzzz"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn media_canvas_notes_filtered_from_find() {
    // media/ and canvas/ notes are excluded by our path filter wrappers
    zettel()
        .args(["--format", "json", "find", "media-ignored"])
        .assert()
        .failure()
        .code(1);

    zettel()
        .args(["--format", "json", "find", "canvas-ignored"])
        .assert()
        .failure()
        .code(1);
}
