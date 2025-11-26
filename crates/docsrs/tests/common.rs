pub fn run_cli(args: &[&str]) -> (String, String, bool) {
    match mx_docsrs::run_cli(args) {
        Ok(stdout) => (stdout, String::new(), true),
        Err(stderr) => (String::new(), stderr, false),
    }
}
