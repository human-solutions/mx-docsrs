pub fn run_cli(args: &[&str]) -> (String, String, bool) {
    // Disable colors for consistent test output
    colored::control::set_override(false);

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
