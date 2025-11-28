pub fn run_cli(args: &[&str]) -> (String, String, bool) {
    match mx_docsrs::run_cli(args) {
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
        } else if let Some(rest) = line.strip_prefix("Using local dependency at /") {
            // Replace "Using local dependency at /path/to/crate (version X.Y.Z)"
            // Find " (version " to extract the version part
            if let Some(version_start) = rest.find(" (version ") {
                let version_part = &rest[version_start..]; // " (version X.Y.Z)"
                format!("Using local dependency at [LOCAL_PATH]{version_part}")
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
