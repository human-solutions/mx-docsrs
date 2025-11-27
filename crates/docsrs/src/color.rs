/// Controls when to use colors in output.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum Color {
    /// Colors will be used if stdout is a terminal. Colors will not be used if
    /// stdout is a regular file.
    #[default]
    Auto,

    /// Colors will never be used.
    Never,

    /// Colors will always be used.
    Always,
}

impl std::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "never" => Ok(Self::Never),
            "always" => Ok(Self::Always),
            _ => Err(format!("Invalid color option: {}", s)),
        }
    }
}
