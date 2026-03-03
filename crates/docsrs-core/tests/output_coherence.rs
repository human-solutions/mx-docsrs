mod common;

use common::run_cli;
use insta::assert_snapshot;

// =============================================================================
// Group A: Crate Root Output
// =============================================================================

#[test]
fn crate_root_shows_doc_and_children() {
    let (stdout, stderr, success) = run_cli(&["test-coherence"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // showing mod test_coherence (crate root)

    /// A test crate for output coherence testing.
    ///
    /// This crate provides various Rust item types to verify that the docsrs
    /// tool produces consistent, well-formed output across all query modes.
    pub mod test_coherence

    pub struct Container
    pub struct Error
    pub const MAX_SIZE
    pub trait Processor
    pub type Result
    pub enum Status
    pub fn process
    pub mod utils
    ");
}

// =============================================================================
// Group B: Path Lookup — Single Item Documentation
// =============================================================================

#[test]
fn path_lookup_struct() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::Container"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)
    // found struct test_coherence::Container

    /// A generic container with public fields and methods.
    ///
    /// Examples
    ///
    ///   let c = Container::new("hello".into(), 42);
    pub struct test_coherence::Container<T> {
        /// The stored value.
        pub value: T,
        /// An optional label for the container.
        pub label: String,
    }

    /* ======== Methods ======== */
    /// Creates a new `Container` with the given value and label.
    pub fn new(value: T, label: String) -> Self
    /// Returns a reference to the stored value.
    pub fn get(&self) -> &T
    "#);
}

#[test]
fn path_lookup_enum() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::Status"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found enum test_coherence::Status

    /// Represents the status of an operation.
    pub enum test_coherence::Status {
        /// The operation has not started.
        Pending,
        /// The operation is running with a progress percentage.
        Running(u8),
        /// The operation completed with a result message.
        Done { message: String },
    }
    ");
}

#[test]
fn path_lookup_trait() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::Processor"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found trait test_coherence::Processor

    /// A trait for processing items.
    ///
    /// Implementors define how items of type [`Self::Input`] are transformed
    /// into [`Self::Output`].
    pub trait test_coherence::Processor {
        /// The input type.
        type Input;
        /// The output type.
        type Output;
        /// The default batch size.
        const DEFAULT_BATCH_SIZE: usize;
        /// Processes a single item.
        fn process(&self, input: Self::Input) -> Self::Output;
        /// Processes a batch of items using the default implementation.
        fn process_batch(&self, items: Vec<Self::Input>) -> Vec<Self::Output> { .. }
    }
    ");
}

#[test]
fn path_lookup_function() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::process"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found fn test_coherence::process

    /// Processes the input value, applying the given transformation.
    ///
    /// This function accepts any type that implements `Into<String>`.
    pub fn test_coherence::process<T>(input: T) -> String where T: Into<String>
    ");
}

#[test]
fn path_lookup_module() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::utils"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found mod test_coherence::utils

    /// Utility functions for common operations.
    pub mod test_coherence::utils

    pub const DEFAULT_BUFFER_SIZE
    pub fn format_debug
    pub mod helpers
    ");
}

#[test]
fn path_lookup_const() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::MAX_SIZE"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found const test_coherence::MAX_SIZE

    /// The maximum allowed size for a container.
    pub const test_coherence::MAX_SIZE: usize
    ");
}

#[test]
fn path_lookup_type_alias() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::Result"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found type test_coherence::Result

    /// A type alias for results with [`Error`].
    pub type test_coherence::Result<T> = Result<T, test_coherence::Error>
    ");
}

#[test]
fn path_lookup_nested() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::utils::helpers::helper_fn"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found fn test_coherence::utils::helpers::helper_fn

    /// A helper function that returns a greeting.
    pub fn test_coherence::utils::helpers::helper_fn(name: &str) -> String
    ");
}

#[test]
fn path_lookup_nonexistent() {
    let (_stdout, stderr, success) = run_cli(&["test-coherence::NonExistent"]);
    assert!(!success, "CLI should fail for nonexistent path");
    assert_snapshot!(stderr, @"No item found at test_coherence::NonExistent");
}

// =============================================================================
// Group C: Filter Search
// =============================================================================

#[test]
fn filter_single_match_returns_doc() {
    let (stdout, stderr, success) = run_cli(&["test-coherence", "Container"]);
    assert!(success, "CLI should succeed: {stderr}");
    // Single exact match returns full documentation, not a list
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)
    // found struct test_coherence::Container

    /// A generic container with public fields and methods.
    ///
    /// Examples
    ///
    ///   let c = Container::new("hello".into(), 42);
    pub struct test_coherence::Container<T> {
        /// The stored value.
        pub value: T,
        /// An optional label for the container.
        pub label: String,
    }

    /* ======== Methods ======== */
    /// Creates a new `Container` with the given value and label.
    pub fn new(value: T, label: String) -> Self
    /// Returns a reference to the stored value.
    pub fn get(&self) -> &T
    "#);
}

#[test]
fn filter_multiple_matches_returns_list() {
    let (stdout, stderr, success) = run_cli(&["test-coherence", "process"]);
    assert!(success, "CLI should succeed: {stderr}");
    // "process" substring matches multiple items → returns sorted list
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)
    // 3 items matching "process"

    fn test_coherence::Processor::process
    fn test_coherence::Processor::process_batch
    fn test_coherence::process
    "#);
}

#[test]
fn filter_no_match_returns_full_list() {
    let (stdout, stderr, success) = run_cli(&["test-coherence", "zzz_nonexistent"]);
    assert!(
        success,
        "CLI should succeed (no results is not an error): {stderr}"
    );
    // No matches → falls back to showing all items in the crate
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)
    // no matches for "zzz_nonexistent" — showing all 15 items

    mod test_coherence
    struct test_coherence::Container
    struct test_coherence::Error
    const test_coherence::MAX_SIZE
    trait test_coherence::Processor
    fn test_coherence::Processor::process
    fn test_coherence::Processor::process_batch
    type test_coherence::Result
    enum test_coherence::Status
    fn test_coherence::process
    mod test_coherence::utils
    const test_coherence::utils::DEFAULT_BUFFER_SIZE
    fn test_coherence::utils::format_debug
    mod test_coherence::utils::helpers
    fn test_coherence::utils::helpers::helper_fn
    "#);
}

#[test]
fn filter_exact_match_returns_doc() {
    let (stdout, stderr, success) = run_cli(&["test-coherence", "Processor"]);
    assert!(success, "CLI should succeed: {stderr}");
    // Exact suffix match on "Processor" returns full trait doc
    assert_snapshot!(stdout, @r"
    // version 0.1.0 (local)
    // found trait test_coherence::Processor

    /// A trait for processing items.
    ///
    /// Implementors define how items of type [`Self::Input`] are transformed
    /// into [`Self::Output`].
    pub trait test_coherence::Processor {
        /// The input type.
        type Input;
        /// The output type.
        type Output;
        /// The default batch size.
        const DEFAULT_BATCH_SIZE: usize;
        /// Processes a single item.
        fn process(&self, input: Self::Input) -> Self::Output;
        /// Processes a batch of items using the default implementation.
        fn process_batch(&self, items: Vec<Self::Input>) -> Vec<Self::Output> { .. }
    }
    ");
}

// =============================================================================
// Group D: Path + Filter (scoped search)
// =============================================================================

#[test]
fn path_filter_within_module() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::utils", "helper"]);
    assert!(success, "CLI should succeed: {stderr}");
    // Searches only within the utils module scope
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)
    // 2 items matching "helper"

    mod test_coherence::utils::helpers
    fn test_coherence::utils::helpers::helper_fn
    "#);
}

#[test]
fn path_filter_no_match_in_scope() {
    let (stdout, stderr, success) = run_cli(&["test-coherence::utils", "Container"]);
    assert!(success, "CLI should succeed: {stderr}");
    // Container is not in utils → falls back to showing all items in utils scope
    assert_snapshot!(stdout, @r#"
    // version 0.1.0 (local)
    // no matches for "Container" — showing all 5 items

    mod test_coherence::utils
    const test_coherence::utils::DEFAULT_BUFFER_SIZE
    fn test_coherence::utils::format_debug
    mod test_coherence::utils::helpers
    fn test_coherence::utils::helpers::helper_fn
    "#);
}

// =============================================================================
// Group E: Query Equivalence
// =============================================================================

#[test]
fn equivalence_struct_path_vs_filter() {
    let (path_stdout, _, path_ok) = run_cli(&["test-coherence::Container"]);
    let (filter_stdout, _, filter_ok) = run_cli(&["test-coherence", "Container"]);
    assert!(path_ok, "Path lookup should succeed");
    assert!(filter_ok, "Filter lookup should succeed");
    assert_eq!(
        path_stdout, filter_stdout,
        "Path and filter lookups should produce identical output for Container"
    );
}

#[test]
fn equivalence_function_path_vs_filter() {
    let (path_stdout, _, path_ok) = run_cli(&["test-coherence::process"]);
    let (filter_stdout, _, filter_ok) = run_cli(&["test-coherence", "process"]);
    assert!(path_ok, "Path lookup should succeed");
    assert!(filter_ok, "Filter lookup should succeed");
    // Path lookup returns single doc; filter "process" matches multiple items (list).
    // These are different query modes, so they may produce different output.
    // When filter has an exact match it returns doc; "process" is an exact suffix
    // match but there are also substring matches, so filter returns a list.
    // This documents the expected difference.
    assert_ne!(
        path_stdout, filter_stdout,
        "Path lookup returns single doc; filter returns list when multiple matches exist"
    );
}

#[test]
fn equivalence_trait_path_vs_filter() {
    let (path_stdout, _, path_ok) = run_cli(&["test-coherence::Processor"]);
    let (filter_stdout, _, filter_ok) = run_cli(&["test-coherence", "Processor"]);
    assert!(path_ok, "Path lookup should succeed");
    assert!(filter_ok, "Filter lookup should succeed");
    assert_eq!(
        path_stdout, filter_stdout,
        "Path and filter lookups should produce identical output for Processor"
    );
}

// =============================================================================
// Group F: List Output Coherence
// =============================================================================

#[test]
fn list_is_sorted_alphabetically() {
    let (stdout, _, success) = run_cli(&["test-coherence", "zzz_nonexistent"]);
    assert!(success);
    let paths: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .map(|l| l.split_whitespace().last().unwrap())
        .collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted, "List items should be sorted alphabetically");
}

#[test]
fn list_has_correct_kind_labels() {
    let (stdout, _, success) = run_cli(&["test-coherence", "zzz_nonexistent"]);
    assert!(success);
    let kinds: Vec<(&str, &str)> = stdout
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            (parts[0], *parts.last().unwrap())
        })
        .collect();

    // Verify specific items have the correct kind label
    for (kind, path) in &kinds {
        match *path {
            "test_coherence" => assert_eq!(*kind, "mod"),
            "test_coherence::Container" => assert_eq!(*kind, "struct"),
            "test_coherence::Error" => assert_eq!(*kind, "struct"),
            "test_coherence::MAX_SIZE" => assert_eq!(*kind, "const"),
            "test_coherence::Processor" => assert_eq!(*kind, "trait"),
            "test_coherence::Result" => assert_eq!(*kind, "type"),
            "test_coherence::Status" => assert_eq!(*kind, "enum"),
            "test_coherence::process" => assert_eq!(*kind, "fn"),
            "test_coherence::utils" => assert_eq!(*kind, "mod"),
            _ => {} // other items are fine
        }
    }
}

#[test]
fn list_no_duplicates() {
    let (stdout, _, success) = run_cli(&["test-coherence", "zzz_nonexistent"]);
    assert!(success);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect();
    let unique_count = {
        let mut seen = std::collections::HashSet::new();
        lines.iter().filter(|l| seen.insert(**l)).count()
    };
    assert_eq!(
        lines.len(),
        unique_count,
        "List should contain no duplicate entries"
    );
}

// =============================================================================
// Group G: External Crate Tests (network-dependent)
// =============================================================================

#[test]
#[ignore = "requires network access"]
fn external_crate_root() {
    let (stdout, stderr, success) = run_cli(&["serde@1.0.228"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.contains("serde"),
        "Output should mention the serde crate"
    );
    // Should show crate root doc and children
    assert!(
        stdout.contains("Serialize") || stdout.contains("Deserialize"),
        "Serde crate root should list key items"
    );
}

#[test]
#[ignore = "requires network access"]
fn external_path_lookup() {
    let (stdout, stderr, success) = run_cli(&["serde@1.0.228::Deserialize"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.contains("trait") || stdout.contains("Deserialize"),
        "Should show Deserialize documentation"
    );
}

#[test]
#[ignore = "requires network access"]
fn external_filter_search() {
    let (stdout, stderr, success) = run_cli(&["serde@1.0.228", "Serialize"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.contains("Serialize"),
        "Filter should find Serialize items"
    );
}

#[test]
#[ignore = "requires network access"]
fn external_path_with_filter() {
    let (stdout, stderr, success) = run_cli(&["serde@1.0.228::de", "Visitor"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.contains("Visitor"),
        "Scoped filter should find Visitor in de module"
    );
}

#[test]
#[ignore = "requires network access"]
fn external_deep_path() {
    let (stdout, stderr, success) = run_cli(&["serde@1.0.228::de::Visitor"]);
    assert!(success, "CLI should succeed: {stderr}");
    assert!(
        stdout.contains("Visitor"),
        "Deep path should resolve to Visitor"
    );
}

// =============================================================================
// Group H: CLI Flag Behavior
// =============================================================================

#[test]
fn no_cache_flag_works() {
    let (stdout, stderr, success) = run_cli(&["--no-cache", "test-coherence"]);
    assert!(success, "CLI should succeed with --no-cache: {stderr}");
    assert!(
        stdout.contains("test_coherence"),
        "Output should still contain crate content"
    );
}

#[test]
fn color_never_flag() {
    let (stdout, stderr, success) = run_cli(&["--color", "never", "test-coherence::Container"]);
    assert!(success, "CLI should succeed: {stderr}");
    // ANSI escape codes start with \x1b[
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI color codes"
    );
    // Should still contain the actual content
    assert!(
        stdout.contains("Container"),
        "Output should still show Container documentation"
    );
}
