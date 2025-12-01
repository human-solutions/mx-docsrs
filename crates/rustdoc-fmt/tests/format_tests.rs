//! Snapshot tests for rustdoc-fmt formatting
//!
//! Tests markdown formatting using inline fixtures that isolate specific features.

use rustdoc_fmt::{DefaultLinkResolver, format_markdown};

fn format_test(markdown: &str) -> String {
    colored::control::set_override(false);
    let result = format_markdown(markdown, &DefaultLinkResolver);
    colored::control::unset_override();
    result
}

// ============================================================================
// Code Blocks
// ============================================================================

#[test]
fn code_block_rust_basic() {
    let markdown = r#"Simple Rust code block:

```rust
fn main() {
    println!("Hello, world!");
}
```
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r#"
    Simple Rust code block:

      fn main() {
          println!("Hello, world!");
      }
    "#);
}

#[test]
fn code_block_rust_hidden_lines() {
    let markdown = r#"Example with hidden lines:

```rust
# fn main() {
let x = 42;
println!("{}", x);
# }
```
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r#"
    Example with hidden lines:

      let x = 42;
      println!("{}", x);
    "#);
}

#[test]
fn code_block_rust_attributes() {
    let markdown = r#"Different code block attributes:

```rust,no_run
fn expensive_operation() {
    // This won't be run during doc tests
}
```

```rust,ignore
broken_code(
```

```rust,should_panic
panic!("expected panic");
```

```rust,compile_fail
let x: i32 = "not a number";
```
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r#"
    Different code block attributes:

      fn expensive_operation() {
          // This won't be run during doc tests
      }
      broken_code(
      panic!("expected panic");
      let x: i32 = "not a number";
    "#);
}

#[test]
fn code_block_other_languages() {
    let markdown = r#"Non-Rust code blocks:

```json
{"key": "value", "number": 42}
```

```bash
cargo build --release
```

```toml
[package]
name = "example"
version = "0.1.0"
```

```
Plain text without language tag
```
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r#"
    Non-Rust code blocks:

      {"key": "value", "number": 42}
      cargo build --release
      [package]
      name = "example"
      version = "0.1.0"
      Plain text without language tag
    "#);
}

// ============================================================================
// Lists
// ============================================================================

#[test]
fn list_unordered_basic() {
    let markdown = r#"Unordered list:

- First item
- Second item
- Third item with `inline code`
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Unordered list:

      • First item
      • Second item
      • Third item with `inline code`
    ");
}

#[test]
fn list_ordered_basic() {
    let markdown = r#"Ordered list:

1. First step
2. Second step
3. Third step
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Ordered list:

      1. First step
      2. Second step
      3. Third step
    ");
}

#[test]
fn list_nested_basic() {
    let markdown = r#"Nested list:

- Level 1 item
  - Level 2 item
    - Level 3 item
  - Another level 2
- Back to level 1
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Nested list:

      • Level 1 item
        ◦ Level 2 item
          ▪ Level 3 item

        ◦ Another level 2

      • Back to level 1
    ");
}

#[test]
fn list_nested_real_async_std() {
    let markdown = r#"Returns `true` if the `Path` has a root.

* On Unix, a path has a root if it begins with `/`.

* On Windows, a path has a root if it:
    * has no prefix and begins with a separator, e.g. `\windows`
    * has a prefix followed by a separator, e.g. `c:\windows` but not `c:windows`
    * has any non-disk prefix, e.g. `\\server\share`
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Returns `true` if the `Path` has a root.

      • On Unix, a path has a root if it begins with `/`.


      • On Windows, a path has a root if it:


        ◦ has no prefix and begins with a separator, e.g. `\windows`
        ◦ has a prefix followed by a separator, e.g. `c:\windows` but not `c:windows`
        ◦ has any non-disk prefix, e.g. `\\server\share`
    ");
}

// ============================================================================
// Links
// ============================================================================

#[test]
fn link_inline_basic() {
    let markdown = r#"Inline links:

See the [Rust documentation](https://doc.rust-lang.org/) for more information.

Also check [this guide](https://example.com/guide).
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Inline links:

    See the Rust documentation (https://doc.rust-lang.org/) for more information.

    Also check this guide (https://example.com/guide).
    ");
}

#[test]
fn link_reference_style() {
    let markdown = r#"Reference-style links:

This crate uses [SwissTable] hash maps for performance.

See the [original implementation][here] and this [CppCon talk].

[SwissTable]: https://abseil.io/blog/20180927-swisstables
[here]: https://github.com/example/repo
[CppCon talk]: https://www.youtube.com/watch?v=example
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Reference-style links:

    This crate uses SwissTable (https://abseil.io/blog/20180927-swisstables) hash maps for performance.

    See the original implementation (https://github.com/example/repo) and this CppCon talk (https://www.youtube.com/watch?v=example).
    ");
}

#[test]
fn link_intra_doc() {
    let markdown = r#"Intra-doc link styles:

See [`Option`] for optional values.
Also check [`Vec`] and [`String`].

Use [`Result::unwrap`] for panic on error.
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Intra-doc link styles:

    See [`Option`] for optional values.
    Also check [`Vec`] and [`String`].

    Use [`Result::unwrap`] for panic on error.
    ");
}

// ============================================================================
// Block Quotes
// ============================================================================

#[test]
fn blockquote_basic() {
    let markdown = r#"A block quote:

> This is a quoted block of text.
> It can span multiple lines.
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    A block quote:

    │ This is a quoted block of text.
    │ It can span multiple lines.
    ");
}

#[test]
fn blockquote_rfc_style() {
    let markdown = r#"RFC-style quote from hyper:

You probably don't need this, here is what [RFC 7230 Section 3.2.4.] has to say:

> A server that receives an obs-fold in a request message that is not
> within a message/http container MUST either reject the message by
> sending a 400 (Bad Request), preferably with a representation
> explaining that obsolete line folding is unacceptable.

Default is false.

[RFC 7230 Section 3.2.4.]: https://tools.ietf.org/html/rfc7230#section-3.2.4
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    RFC-style quote from hyper:

    You probably don't need this, here is what RFC 7230 Section 3.2.4. (https://tools.ietf.org/html/rfc7230#section-3.2.4) has to say:

    │ A server that receives an obs-fold in a request message that is not
    │ within a message/http container MUST either reject the message by
    │ sending a 400 (Bad Request), preferably with a representation
    │ explaining that obsolete line folding is unacceptable.

    Default is false.
    ");
}

// ============================================================================
// Headings
// ============================================================================

#[test]
fn heading_all_levels() {
    let markdown = r#"# Heading 1

Content under heading 1.

## Heading 2

Content under heading 2.

### Heading 3

Content under heading 3.

#### Heading 4

Content under heading 4.
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Heading 1

    Content under heading 1.

    Heading 2

    Content under heading 2.

    Heading 3

    Content under heading 3.

    Heading 4

    Content under heading 4.
    ");
}

#[test]
fn heading_with_code() {
    let markdown = r#"# The `main` function

## Using `Option<T>`

### Method `unwrap()`
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    The main function

    Using Option<T>

    Method unwrap()
    ");
}

// ============================================================================
// Emphasis
// ============================================================================

#[test]
fn emphasis_basic() {
    let markdown = r#"Text styling:

This is *italic text* and this is **bold text**.

You can also use ~~strikethrough~~ text.
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Text styling:

    This is italic text and this is bold text.

    You can also use strikethrough text.
    ");
}

#[test]
fn emphasis_nested() {
    let markdown = r#"Nested emphasis:

This is ***bold and italic*** text.

This is **bold with *nested italic* inside**.
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Nested emphasis:

    This is bold and italic text.

    This is nested italicbold with  inside.
    ");
}

// ============================================================================
// Inline Code
// ============================================================================

#[test]
fn inline_code_basic() {
    let markdown = r#"Inline code examples:

Use `Option<T>` for optional values.

Call the `foo()` function with `bar` as argument.

The `HashMap<K, V>` type stores key-value pairs.
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Inline code examples:

    Use `Option<T>` for optional values.

    Call the `foo()` function with `bar` as argument.

    The `HashMap<K, V>` type stores key-value pairs.
    ");
}

// ============================================================================
// Complex / Mixed Features
// ============================================================================

#[test]
fn complex_mixed_features() {
    let markdown = r#"# Getting Started

This is an introduction with **bold** and *italic* text.

## Usage

Use the `example` function:

```rust
let result = example(42);
```

## Options

- Use `--verbose` for verbose output
- Use `--quiet` for quiet mode
  - Also silences warnings

See [`Config`] for more details.
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Getting Started

    This is an introduction with bold and italic text.

    Usage

    Use the `example` function:

      let result = example(42);
    Options

      • Use `--verbose` for verbose output
      • Use `--quiet` for quiet mode
        ◦ Also silences warnings


    See [`Config`] for more details.
    ");
}

#[test]
fn complex_real_num_traits() {
    let markdown = r#"Returns a number that represents the sign of `self`.

- `1.0` if the number is positive, `+0.0` or `Float::infinity()`
- `-1.0` if the number is negative, `-0.0` or `Float::neg_infinity()`
- `Float::nan()` if the number is `Float::nan()`

```rust
use num_traits::Float;
use std::f64;

let f = 3.5;

assert_eq!(f.signum(), 1.0);
assert_eq!(f64::NEG_INFINITY.signum(), -1.0);

assert!(f64::NAN.signum().is_nan());
```
"#;
    let result = format_test(markdown);
    insta::assert_snapshot!(result, @r"
    Returns a number that represents the sign of `self`.

      • `1.0` if the number is positive, `+0.0` or `Float::infinity()`
      • `-1.0` if the number is negative, `-0.0` or `Float::neg_infinity()`
      • `Float::nan()` if the number is `Float::nan()`

      use num_traits::Float;
      use std::f64;
      
      let f = 3.5;
      
      assert_eq!(f.signum(), 1.0);
      assert_eq!(f64::NEG_INFINITY.signum(), -1.0);
      
      assert!(f64::NAN.signum().is_nan());
    ");
}
