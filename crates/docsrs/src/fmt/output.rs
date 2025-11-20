use super::tokens::Token;

/// A builder-style wrapper around `Vec<Token>` for ergonomic token construction.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Output {
    tokens: Vec<Token>,
}

impl Output {
    /// Create a new empty output.
    pub(crate) fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Add a symbol token, like `=` or `::<`.
    pub(crate) fn symbol(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Symbol(text.into()));
        self
    }

    /// Add a qualifier token, like `pub` or `const`.
    pub(crate) fn qualifier(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Qualifier(text.into()));
        self
    }

    /// Add a kind token, like `function` or `trait`.
    pub(crate) fn kind(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Kind(text.into()));
        self
    }

    /// Add whitespace (a single space).
    pub(crate) fn whitespace(&mut self) -> &mut Self {
        self.tokens.push(Token::Whitespace);
        self
    }

    /// Add an identifier token, like variable names or parts of the path of an item.
    pub(crate) fn identifier(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Identifier(text.into()));
        self
    }

    /// Add an annotation token, used e.g. for Rust attributes.
    pub(crate) fn annotation(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Annotation(text.into()));
        self
    }

    /// Add a self token, the text can be `self` or `Self`.
    pub(crate) fn self_(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Self_(text.into()));
        self
    }

    /// Add a function identifier token.
    pub(crate) fn function(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Function(text.into()));
        self
    }

    /// Add a lifetime token including the apostrophe `'`, like `'a`.
    pub(crate) fn lifetime(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Lifetime(text.into()));
        self
    }

    /// Add a keyword token, like `impl`, `where`, or `dyn`.
    pub(crate) fn keyword(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Keyword(text.into()));
        self
    }

    /// Add a generic parameter token, like `T`.
    pub(crate) fn generic(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Generic(text.into()));
        self
    }

    /// Add a primitive type token, like `usize`.
    pub(crate) fn primitive(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Primitive(text.into()));
        self
    }

    /// Add a type token, like the name of a struct or a trait.
    pub(crate) fn type_(&mut self, text: impl Into<String>) -> &mut Self {
        self.tokens.push(Token::Type(text.into()));
        self
    }

    /// Extend this output with all tokens from another output.
    pub(crate) fn extend(&mut self, other: Output) -> &mut Self {
        self.tokens.extend(other.tokens);
        self
    }

    /// Remove the last token that was added.
    pub(crate) fn pop(&mut self) {
        self.tokens.pop();
    }

    /// Convert this output into a vector of tokens.
    pub(crate) fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    /// Get a reference to the underlying tokens.
    pub(crate) fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    // Convenience methods for common token sequences

    /// Add "pub " (qualifier + whitespace).
    pub(crate) fn qualifier_pub(mut self) -> Self {
        self.qualifier("pub").whitespace();
        self
    }

    /// Add " + " (whitespace + symbol + whitespace).
    pub(crate) fn symbol_plus(mut self) -> Self {
        self.whitespace().symbol("+").whitespace();
        self
    }

    /// Add ": " (symbol + whitespace).
    pub(crate) fn symbol_colon(mut self) -> Self {
        self.symbol(":").whitespace();
        self
    }

    /// Add ", " (symbol + whitespace).
    pub(crate) fn symbol_comma(mut self) -> Self {
        self.symbol(",").whitespace();
        self
    }

    /// Add " = " (whitespace + symbol + whitespace).
    pub(crate) fn symbol_equals(mut self) -> Self {
        self.whitespace().symbol("=").whitespace();
        self
    }

    /// Add " -> " (whitespace + symbol + whitespace).
    pub(crate) fn symbol_arrow(mut self) -> Self {
        self.whitespace().symbol("->").whitespace();
        self
    }
}
