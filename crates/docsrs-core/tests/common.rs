pub fn run_cli(args: &[&str]) -> (String, String, bool) {
    // Disable colors for consistent test output
    colored::control::set_override(false);

    match docsrs_core::run_cli(args) {
        Ok(stdout) => (normalize_output(&stdout), String::new(), true),
        Err(stderr) => (String::new(), stderr, false),
    }
}

/// Variant that does NOT override colors — for testing the --color flag,
/// where forcing colors off would defeat the purpose. Each test is responsible
/// for managing the global colored::control::set_override state.
#[allow(dead_code)]
pub fn run_cli_raw(args: &[&str]) -> (String, String, bool) {
    match docsrs_core::run_cli(args) {
        Ok(stdout) => (normalize_output(&stdout), String::new(), true),
        Err(stderr) => (String::new(), stderr, false),
    }
}

/// Normalize output by replacing machine-specific paths with placeholders
fn normalize_output(output: &str) -> String {
    // No machine-specific paths in the new comment format
    output.to_string()
}
