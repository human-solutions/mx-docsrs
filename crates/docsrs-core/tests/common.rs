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
    let mut lines: Vec<String> = Vec::new();

    for line in output.lines() {
        let normalized = if line.starts_with("Local crate found at: ") {
            // Replace "Local crate found at: /path/to/file.json"
            "Local crate found at: [LOCAL_PATH]".to_string()
        } else if let Some(rest) = line.strip_prefix("Using local dependency version ") {
            // Replace "Using local dependency version X.Y.Z at /path/to/crate"
            if let Some(at_idx) = rest.find(" at /") {
                let version = &rest[..at_idx];
                format!("Using local dependency version {version} at [LOCAL_PATH]")
            } else {
                line.to_string()
            }
        } else {
            line.to_string()
        };
        lines.push(normalized);
    }

    let mut result = lines.join("\n");
    // Preserve trailing newline if original had one
    if output.ends_with('\n') {
        result.push('\n');
    }
    result
}
