use std::process::Command;

fn main() {
    let pkg_version = std::env::var("CARGO_PKG_VERSION").unwrap();

    let sha = Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=DOCSRS_BUILD_VERSION={pkg_version}+{sha}");
    println!("cargo:rerun-if-changed=build.rs");
    // Reflog updates on every HEAD movement (commit, checkout, reset).
    println!("cargo:rerun-if-changed=../../.git/logs/HEAD");
    // Captures branch switches that don't move the reflog.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
}
