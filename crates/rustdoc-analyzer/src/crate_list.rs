//! List of crates to analyze for markdown patterns

/// Category of a crate for analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CrateCategory {
    StdLibrary,
    Async,
    Web,
    Cli,
    Parsing,
    Crypto,
    DataStructures,
    ErrorHandling,
    Utilities,
}

impl CrateCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            CrateCategory::StdLibrary => "std_library",
            CrateCategory::Async => "async",
            CrateCategory::Web => "web",
            CrateCategory::Cli => "cli",
            CrateCategory::Parsing => "parsing",
            CrateCategory::Crypto => "crypto",
            CrateCategory::DataStructures => "data_structures",
            CrateCategory::ErrorHandling => "error_handling",
            CrateCategory::Utilities => "utilities",
        }
    }
}

/// Information about a crate to analyze
#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub category: CrateCategory,
}

/// All crates to analyze (~50 crates across categories)
pub static CRATES: &[CrateInfo] = &[
    // Std Library (5)
    CrateInfo {
        name: "hashbrown",
        version: "latest",
        category: CrateCategory::StdLibrary,
    },
    CrateInfo {
        name: "libc",
        version: "latest",
        category: CrateCategory::StdLibrary,
    },
    CrateInfo {
        name: "core-foundation",
        version: "latest",
        category: CrateCategory::StdLibrary,
    },
    CrateInfo {
        name: "num-traits",
        version: "latest",
        category: CrateCategory::StdLibrary,
    },
    CrateInfo {
        name: "bitflags",
        version: "latest",
        category: CrateCategory::StdLibrary,
    },
    // Async (5)
    CrateInfo {
        name: "tokio",
        version: "latest",
        category: CrateCategory::Async,
    },
    CrateInfo {
        name: "async-std",
        version: "latest",
        category: CrateCategory::Async,
    },
    CrateInfo {
        name: "futures",
        version: "latest",
        category: CrateCategory::Async,
    },
    CrateInfo {
        name: "smol",
        version: "latest",
        category: CrateCategory::Async,
    },
    CrateInfo {
        name: "async-trait",
        version: "latest",
        category: CrateCategory::Async,
    },
    // Web (5)
    CrateInfo {
        name: "axum",
        version: "latest",
        category: CrateCategory::Web,
    },
    CrateInfo {
        name: "actix-web",
        version: "latest",
        category: CrateCategory::Web,
    },
    CrateInfo {
        name: "warp",
        version: "latest",
        category: CrateCategory::Web,
    },
    CrateInfo {
        name: "rocket",
        version: "latest",
        category: CrateCategory::Web,
    },
    CrateInfo {
        name: "hyper",
        version: "latest",
        category: CrateCategory::Web,
    },
    // CLI (5)
    CrateInfo {
        name: "clap",
        version: "latest",
        category: CrateCategory::Cli,
    },
    CrateInfo {
        name: "structopt",
        version: "latest",
        category: CrateCategory::Cli,
    },
    CrateInfo {
        name: "argh",
        version: "latest",
        category: CrateCategory::Cli,
    },
    CrateInfo {
        name: "dialoguer",
        version: "latest",
        category: CrateCategory::Cli,
    },
    CrateInfo {
        name: "indicatif",
        version: "latest",
        category: CrateCategory::Cli,
    },
    // Parsing (5)
    CrateInfo {
        name: "serde",
        version: "latest",
        category: CrateCategory::Parsing,
    },
    CrateInfo {
        name: "serde_json",
        version: "latest",
        category: CrateCategory::Parsing,
    },
    CrateInfo {
        name: "nom",
        version: "latest",
        category: CrateCategory::Parsing,
    },
    CrateInfo {
        name: "pest",
        version: "latest",
        category: CrateCategory::Parsing,
    },
    CrateInfo {
        name: "toml",
        version: "latest",
        category: CrateCategory::Parsing,
    },
    // Crypto (5)
    CrateInfo {
        name: "ring",
        version: "latest",
        category: CrateCategory::Crypto,
    },
    CrateInfo {
        name: "rustls",
        version: "latest",
        category: CrateCategory::Crypto,
    },
    CrateInfo {
        name: "sha2",
        version: "latest",
        category: CrateCategory::Crypto,
    },
    CrateInfo {
        name: "aes",
        version: "latest",
        category: CrateCategory::Crypto,
    },
    CrateInfo {
        name: "rand",
        version: "latest",
        category: CrateCategory::Crypto,
    },
    // Data Structures (5)
    CrateInfo {
        name: "indexmap",
        version: "latest",
        category: CrateCategory::DataStructures,
    },
    CrateInfo {
        name: "smallvec",
        version: "latest",
        category: CrateCategory::DataStructures,
    },
    CrateInfo {
        name: "bytes",
        version: "latest",
        category: CrateCategory::DataStructures,
    },
    CrateInfo {
        name: "dashmap",
        version: "latest",
        category: CrateCategory::DataStructures,
    },
    CrateInfo {
        name: "petgraph",
        version: "latest",
        category: CrateCategory::DataStructures,
    },
    // Error Handling (5)
    CrateInfo {
        name: "anyhow",
        version: "latest",
        category: CrateCategory::ErrorHandling,
    },
    CrateInfo {
        name: "thiserror",
        version: "latest",
        category: CrateCategory::ErrorHandling,
    },
    CrateInfo {
        name: "eyre",
        version: "latest",
        category: CrateCategory::ErrorHandling,
    },
    CrateInfo {
        name: "color-eyre",
        version: "latest",
        category: CrateCategory::ErrorHandling,
    },
    CrateInfo {
        name: "miette",
        version: "latest",
        category: CrateCategory::ErrorHandling,
    },
    // Utilities (10)
    CrateInfo {
        name: "itertools",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "regex",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "once_cell",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "tracing",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "log",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "chrono",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "uuid",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "url",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "crossbeam",
        version: "latest",
        category: CrateCategory::Utilities,
    },
    CrateInfo {
        name: "rayon",
        version: "latest",
        category: CrateCategory::Utilities,
    },
];

/// Get crates filtered by category
pub fn crates_by_category(category: CrateCategory) -> impl Iterator<Item = &'static CrateInfo> {
    CRATES.iter().filter(move |c| c.category == category)
}

/// Get all categories
pub fn all_categories() -> impl Iterator<Item = CrateCategory> {
    [
        CrateCategory::StdLibrary,
        CrateCategory::Async,
        CrateCategory::Web,
        CrateCategory::Cli,
        CrateCategory::Parsing,
        CrateCategory::Crypto,
        CrateCategory::DataStructures,
        CrateCategory::ErrorHandling,
        CrateCategory::Utilities,
    ]
    .into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_count() {
        assert_eq!(CRATES.len(), 50);
    }

    #[test]
    fn test_category_distribution() {
        let std_count = crates_by_category(CrateCategory::StdLibrary).count();
        let async_count = crates_by_category(CrateCategory::Async).count();
        let utilities_count = crates_by_category(CrateCategory::Utilities).count();

        assert_eq!(std_count, 5);
        assert_eq!(async_count, 5);
        assert_eq!(utilities_count, 10);
    }
}
