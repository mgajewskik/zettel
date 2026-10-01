use std::path::PathBuf;
use std::process::Command;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;

fn check_links_vault() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/check_links_vault")
}

fn zettel_check() -> Command {
    let mut cmd = Command::cargo_bin("zettel").unwrap();
    cmd.arg("--vault").arg(check_links_vault());
    cmd
}

#[test]
fn check_links_flags_permanent_to_source() {
    // Permanent at root linking [[source/foo]] → flagged; exit 1
    zettel_check()
        .args(["check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("perm-to-source"))
        .stdout(predicate::str::contains("source/foo"));
}

#[test]
fn check_links_source_to_permanent_not_flagged() {
    let assert = zettel_check()
        .args(["--format", "json", "check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    // source/foo.md may appear as resolved_path; it must never be the violation source
    assert!(
        !stdout.contains("\"source_path\": \"source/"),
        "source/** must not appear as violation sources: {stdout}"
    );
}

#[test]
fn check_links_permanent_to_permanent_not_flagged() {
    let assert = zettel_check()
        .args(["--format", "json", "check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    // perm-ok only links another permanent — must not appear as a violation source
    assert!(
        !stdout.contains("\"source_path\": \"perm-ok.md\""),
        "permanent→permanent must not be flagged: {stdout}"
    );
}

#[test]
fn check_links_json_fields() {
    zettel_check()
        .args(["--format", "json", "check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("\"target\""))
        .stdout(predicate::str::contains("\"source_path\""))
        .stdout(predicate::str::contains("\"resolved_path\""));
}

#[test]
fn check_links_inbox_not_flagged() {
    let assert = zettel_check()
        .args(["--format", "json", "check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains("inbox/"),
        "inbox sources are not permanent: {stdout}"
    );
}

#[test]
fn check_links_area_to_source_not_flagged() {
    let assert = zettel_check()
        .args(["--format", "json", "check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1);
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(
        !stdout.contains("area/"),
        "area/** must not be flagged as permanent→source: {stdout}"
    );
}

#[test]
fn check_links_root_moc_to_source_flagged() {
    zettel_check()
        .args(["check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("moc-overview"))
        .stdout(predicate::str::contains("source/foo"));
}

#[test]
fn check_links_basename_resolving_to_source() {
    zettel_check()
        .args(["check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("perm-basename-src"))
        .stdout(predicate::str::contains("unique-src"));
}

#[test]
fn check_links_markdown_to_source() {
    zettel_check()
        .args(["check-links", "--rule", "zettel"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("perm-md-link"))
        .stdout(predicate::str::contains("source/foo"));
}

#[test]
fn check_links_path_filter() {
    // Only check perm-ok (clean) → exit 0
    zettel_check()
        .args(["check-links", "--rule", "zettel", "perm-ok"])
        .assert()
        .success()
        .code(0);
}

#[test]
fn check_links_scope_root_default() {
    // Explicit --scope root matches default behavior (still finds violations)
    zettel_check()
        .args(["check-links", "--rule", "zettel", "--scope", "root"])
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::contains("perm-to-source"));
}

#[test]
fn check_links_requires_rule() {
    zettel_check()
        .args(["check-links"])
        .assert()
        .failure(); // clap missing required --rule
}

#[test]
fn check_links_missing_vault_exit_2() {
    let mut cmd = Command::cargo_bin("zettel").unwrap();
    cmd.env_remove("ZETTEL_VAULT");
    cmd.args(["check-links", "--rule", "zettel"]);
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("ZETTEL_VAULT"));
}
