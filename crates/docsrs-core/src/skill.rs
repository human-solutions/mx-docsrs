use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::BaseDirs;

use crate::cli::SkillScope;

pub const SKILL_MD: &str = include_str!("../assets/SKILL.md");

const SKILL_DIR_SUFFIX: &str = ".claude/skills/docsrs";
const SKILL_FILE_NAME: &str = "SKILL.md";

#[derive(Debug, PartialEq, Eq)]
pub enum InstallOutcome {
    Written,
    AlreadyUpToDate,
    Differs,
}

pub fn resolve_scope_dir(scope: SkillScope) -> Result<PathBuf> {
    match scope {
        SkillScope::User => {
            let base = BaseDirs::new().context("Could not determine the user's home directory")?;
            Ok(base.home_dir().join(SKILL_DIR_SUFFIX))
        }
        SkillScope::Project => {
            let cwd = std::env::current_dir()
                .context("Could not determine the current working directory")?;
            Ok(cwd.join(SKILL_DIR_SUFFIX))
        }
    }
}

/// Install the bundled SKILL.md into `target_dir`. Idempotent: returns
/// `AlreadyUpToDate` if the file already matches byte-for-byte; returns
/// `Differs` if it exists but is different (caller should suggest `--force`).
pub fn install_skill(target_dir: &Path, force: bool) -> Result<InstallOutcome> {
    fs::create_dir_all(target_dir)
        .with_context(|| format!("Could not create directory {}", target_dir.display()))?;

    let target = target_dir.join(SKILL_FILE_NAME);

    if target.exists() {
        let existing = fs::read_to_string(&target)
            .with_context(|| format!("Could not read existing {}", target.display()))?;
        if existing == SKILL_MD {
            return Ok(InstallOutcome::AlreadyUpToDate);
        }
        if !force {
            return Ok(InstallOutcome::Differs);
        }
    }

    // Atomic write: tmp file in the same directory, then rename.
    let tmp = target_dir.join(format!("{SKILL_FILE_NAME}.tmp"));
    fs::write(&tmp, SKILL_MD).with_context(|| format!("Could not write {}", tmp.display()))?;
    fs::rename(&tmp, &target)
        .with_context(|| format!("Could not rename {} -> {}", tmp.display(), target.display()))?;

    Ok(InstallOutcome::Written)
}

/// Returns the path of a competing SKILL.md at the *other* scope, if one
/// exists. Used to print a one-line shadowing notice after a successful
/// install. Claude Code precedence is personal (user) > project, so the
/// warning text callers print should explain which copy wins where.
pub fn other_scope_skill(installed_scope: SkillScope) -> Option<PathBuf> {
    let other = match installed_scope {
        SkillScope::User => SkillScope::Project,
        SkillScope::Project => SkillScope::User,
    };
    let dir = resolve_scope_dir(other).ok()?;
    let file = dir.join(SKILL_FILE_NAME);
    file.exists().then_some(file)
}
