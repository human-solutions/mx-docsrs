use clap::Parser;

/// Search for documentation of a symbol in a crate
#[derive(Parser, Debug)]
#[command(name = "docsrs")]
#[command(about = "Search for documentation of a symbol in a crate", long_about = None)]
pub struct Cli {
    /// The crate name to search in
    pub crate_name: String,

    /// The symbol to search for
    pub symbol: String,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
