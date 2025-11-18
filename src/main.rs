mod cli;

use cli::Cli;

fn main() {
    let args = Cli::parse_args();

    println!("Crate: {}", args.crate_name);
    println!("Symbol: {}", args.symbol);
}
