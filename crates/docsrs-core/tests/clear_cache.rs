//! Test for `--clear-cache`. Lives in its own integration-test binary
//! so it runs in its own process and can't race other tests that touch
//! the cache.
//!
//! NOTE: `--clear-cache` wipes the real user cache directory (on macOS:
//! `~/Library/Caches/docsrs/`). This is acceptable on CI but disruptive on
//! local dev machines, so the test is gated with `#[ignore]`. Run it with:
//!
//!     cargo nextest run --workspace --run-ignored only

mod common;

use common::run_cli;

#[test]
#[ignore = "wipes the user's docsrs cache directory; opt-in only"]
fn clear_cache_removes_cache_directory() {
    let (stdout, stderr, success) = run_cli(&["--clear-cache"]);
    assert!(success, "--clear-cache should succeed: {stderr}");
    assert_eq!(stdout, "Cache cleared successfully\n");
}
