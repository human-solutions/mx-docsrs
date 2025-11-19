use std::process;

fn main() {
    // Collect args (skip program name)
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    // Run the CLI and handle result
    match mx_docsrs::run_cli(&args_refs) {
        Ok(output) => {
            print!("{}", output);
            process::exit(0);
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            process::exit(1);
        }
    }
}
