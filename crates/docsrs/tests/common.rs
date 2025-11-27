pub fn run_cli(args: &[&str]) -> (String, String, bool) {
    match mx_docsrs::run_cli(args) {
        Ok(stdout) => (normalize_output(&stdout), String::new(), true),
        Err(stderr) => (String::new(), stderr, false),
    }
}

/// Normalize output by replacing machine-specific paths with placeholders
fn normalize_output(output: &str) -> String {
    // Replace "Local crate found at: /path/to/file.json" with a normalized version
    let mut result = String::new();
    for line in output.lines() {
        if line.starts_with("Local crate found at: ") {
            result.push_str("Local crate found at: [LOCAL_PATH]\n");
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }
    // Remove trailing newline if original didn't have one
    if !output.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}
