//! Demo binary for rustdoc-fmt visual testing.

use rustdoc_fmt::{DefaultLinkResolver, format_markdown};

fn main() {
    let demo_markdown = r#"
# Heading Level 1

This is a paragraph with **bold text** and *italic text*.

## Heading Level 2

Here's some `inline code` and a [link](https://example.com).

### Heading Level 3

A code block with syntax highlighting:

```rust
fn main() {
    let greeting = "Hello, world!";
    println!("{}", greeting);
}
```

#### Heading Level 4

An unordered list:

- First item
- Second item with `code`
- Third item

An ordered list:

1. Step one
2. Step two
3. Step three

A nested list:

- Outer item 1
  - Inner item A
  - Inner item B
- Outer item 2
  - Inner item C
    - Deep item X
    - Deep item Y

> This is a blockquote.
> It can span multiple lines.

HTML content with <b>bold</b> and <em>emphasis</em> tags:

<div class="warning">
This is a warning box with <code>inline code</code>.
</div>

Another code block in a different language:

```json
{
  "name": "example",
  "version": "1.0.0"
}
```
"#;

    println!("=== rustdoc-fmt Demo ===\n");
    let resolver = DefaultLinkResolver;
    let formatted = format_markdown(demo_markdown, &resolver);
    print!("{}", formatted);
}
