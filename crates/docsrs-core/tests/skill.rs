//! Tests for `--print-skill` and `--install-skill`. Lives in its own
//! integration-test binary so it runs in its own process and never races
//! other tests that touch the cache or filesystem.

mod common;

use std::fs;

use docsrs_core::skill::{InstallOutcome, SKILL_MD, install_skill};

use common::run_cli;

#[test]
fn print_skill_emits_bundled_markdown() {
    let (stdout, stderr, success) = run_cli(&["--print-skill"]);
    assert!(success, "--print-skill should succeed: {stderr}");
    assert_eq!(stdout, SKILL_MD);
    assert!(
        stdout.starts_with("---\nname: docsrs\n"),
        "expected YAML frontmatter at the top, got: {}",
        &stdout[..stdout.len().min(80)]
    );
}

#[test]
fn print_skill_conflicts_with_crate_spec() {
    let (_stdout, stderr, success) = run_cli(&["--print-skill", "tokio"]);
    assert!(!success, "expected clap conflict error");
    assert!(
        stderr.contains("--print-skill") || stderr.contains("cannot be used with"),
        "expected a conflict error mentioning --print-skill, got: {stderr}"
    );
}

#[test]
fn install_skill_writes_file_into_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join(".claude/skills/docsrs");
    let outcome = install_skill(&target, false).unwrap();
    assert_eq!(outcome, InstallOutcome::Written);

    let written = fs::read_to_string(target.join("SKILL.md")).unwrap();
    assert_eq!(written, SKILL_MD);
}

#[test]
fn install_skill_creates_nested_parent_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("a/b/c/.claude/skills/docsrs");
    let outcome = install_skill(&target, false).unwrap();
    assert_eq!(outcome, InstallOutcome::Written);
    assert!(target.join("SKILL.md").exists());
}

#[test]
fn install_skill_is_idempotent_when_content_matches() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("skills");
    assert_eq!(
        install_skill(&target, false).unwrap(),
        InstallOutcome::Written
    );
    assert_eq!(
        install_skill(&target, false).unwrap(),
        InstallOutcome::AlreadyUpToDate
    );
}

#[test]
fn install_skill_refuses_to_overwrite_differing_content() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("skills");
    fs::create_dir_all(&target).unwrap();
    let existing = "---\nname: docsrs\n---\nuser-edited content";
    fs::write(target.join("SKILL.md"), existing).unwrap();

    let outcome = install_skill(&target, false).unwrap();
    assert_eq!(outcome, InstallOutcome::Differs);

    let unchanged = fs::read_to_string(target.join("SKILL.md")).unwrap();
    assert_eq!(unchanged, existing, "file must not be modified");
}

#[test]
fn install_skill_force_overwrites_differing_content() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("skills");
    fs::create_dir_all(&target).unwrap();
    fs::write(target.join("SKILL.md"), "old content").unwrap();

    let outcome = install_skill(&target, true).unwrap();
    assert_eq!(outcome, InstallOutcome::Written);

    let after = fs::read_to_string(target.join("SKILL.md")).unwrap();
    assert_eq!(after, SKILL_MD);
}

#[test]
fn install_skill_leaves_no_temp_file_behind() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("skills");
    install_skill(&target, false).unwrap();
    assert!(!target.join("SKILL.md.tmp").exists());
}
