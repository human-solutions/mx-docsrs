//! Contains all token handling logic.

/// A token in a rendered public item, used to apply syntax coloring in downstream applications.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    /// A symbol, like `=` or `::<`
    Symbol(String),
    /// A qualifier, like `pub` or `const`
    Qualifier(String),
    /// The kind of an item, like `function` or `trait`
    Kind(String),
    /// Whitespace, a single space
    Whitespace,
    /// An identifier, like variable names or parts of the path of an item
    Identifier(String),
    /// An annotation, used e.g. for Rust attributes.
    Annotation(String),
    /// The identifier self, the text can be `self` or `Self`
    Self_(String),
    /// The identifier for a function
    Function(String),
    /// A lifetime including the apostrophe `'`, like `'a`
    Lifetime(String),
    /// A keyword, like `impl`, `where`, or `dyn`
    Keyword(String),
    /// A generic parameter, like `T`
    Generic(String),
    /// A primitive type, like `usize`
    Primitive(String),
    /// A non-primitive type, like the name of a struct or a trait
    Type(String),
}

impl Token {
    /// Get the inner text of this token
    pub fn text(&self) -> &str {
        match self {
            Self::Symbol(l)
            | Self::Qualifier(l)
            | Self::Kind(l)
            | Self::Identifier(l)
            | Self::Annotation(l)
            | Self::Self_(l)
            | Self::Function(l)
            | Self::Lifetime(l)
            | Self::Keyword(l)
            | Self::Generic(l)
            | Self::Primitive(l)
            | Self::Type(l) => l,
            Self::Whitespace => " ",
        }
    }
}

/// Convert a slice of tokens to a single string.
pub fn tokens_to_string(tokens: &[Token]) -> String {
    tokens.iter().map(Token::text).collect()
}
