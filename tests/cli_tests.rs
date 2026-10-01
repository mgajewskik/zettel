use std::path::PathBuf;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::assert::OutputAssertExt;
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
fn links_json() {
    zettel()
        .args(["--format", "json", "links", "alpha"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"kind\""))
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
fn unresolved_reports_missing_and_exit_1() {
    zettel()
        .args(["unresolved"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("missing-note"));
}

#[test]
fn unresolved_json() {
    zettel()
        .args(["--format", "json", "unresolved"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"text\""));
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
fn find_matches() {
    zettel()
        .args(["find", "gamma"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gamma"));
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
