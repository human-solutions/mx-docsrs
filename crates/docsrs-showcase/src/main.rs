use std::fmt::Write;

struct Example {
    title: &'static str,
    category: &'static str,
    args: &'static [&'static str],
}

fn main() {
    colored::control::set_override(true);

    let examples: Vec<Example> = vec![
        // --- Re-exports ---
        Example {
            title: "Simple re-export",
            category: "Re-exports",
            args: &["test-reexports::InnerStruct", "--color=always"],
        },
        Example {
            title: "Renamed re-export",
            category: "Re-exports",
            args: &["test-reexports::RenamedStruct", "--color=always"],
        },
        // --- Visibility ---
        Example {
            title: "Public items (non-public excluded)",
            category: "Visibility",
            args: &["test-visibility", "--color=always"],
        },
        // --- Errors ---
        Example {
            title: "Path not found",
            category: "Errors",
            args: &["test-visibility::NonexistentItem", "--color=always"],
        },
        // --- Special ---
        Example {
            title: "Help text",
            category: "Special",
            args: &["--help"],
        },
    ];

    let mut html = String::new();
    write_header(&mut html);

    // Build sidebar and main content
    let categories = collect_categories(&examples);
    write_sidebar(&mut html, &categories, &examples);
    html.push_str("  <main>\n");
    html.push_str("    <h1>docsrs Output Showcase</h1>\n");
    html.push_str(
        "    <p class=\"subtitle\">All output variants with terminal coloring, \
         generated from local test crates.</p>\n",
    );

    for (i, example) in examples.iter().enumerate() {
        let display_cmd = format_display_command(example.args);
        let id = format!("example-{}", i);

        let (output, is_error) = match docsrs_core::run_cli(example.args) {
            Ok(out) => (out, false),
            Err(err) => (err, true),
        };

        let html_output = ansi_to_html::convert(&output)
            .unwrap_or_else(|e| format!("&lt;ANSI conversion error: {}&gt;", e));

        write!(
            html,
            "\n    <section id=\"{id}\">\n      \
             <h2><span class=\"category-tag\">{category}</span> {title}</h2>\n      \
             <div class=\"command\">$ {cmd}{err_tag}</div>\n      \
             <pre class=\"output\">{output}</pre>\n    \
             </section>\n",
            id = id,
            category = example.category,
            title = example.title,
            cmd = display_cmd,
            err_tag = if is_error {
                " <span class=\"error-tag\">ERROR</span>"
            } else {
                ""
            },
            output = html_output,
        )
        .unwrap();

        eprint!(".");
    }

    html.push_str("  </main>\n");
    write_footer(&mut html);

    print!("{html}");
    eprintln!("\nGenerated showcase with {} examples.", examples.len());
}

fn format_display_command(args: &[&str]) -> String {
    let display_args: Vec<&str> = args
        .iter()
        .filter(|a| !a.starts_with("--color"))
        .copied()
        .collect();
    format!("docsrs {}", display_args.join(" "))
}

fn collect_categories(examples: &[Example]) -> Vec<&'static str> {
    let mut cats: Vec<&str> = Vec::new();
    for ex in examples {
        if !cats.contains(&ex.category) {
            cats.push(ex.category);
        }
    }
    cats
}

fn write_sidebar(html: &mut String, categories: &[&str], examples: &[Example]) {
    html.push_str("  <nav id=\"sidebar\">\n");
    html.push_str("    <div class=\"sidebar-header\">docsrs Showcase</div>\n");
    html.push_str("    <ul>\n");
    for cat in categories {
        writeln!(html, "      <li class=\"cat-header\">{cat}</li>").unwrap();
        for (i, ex) in examples.iter().enumerate() {
            if ex.category == *cat {
                writeln!(
                    html,
                    "      <li><a href=\"#example-{i}\">{title}</a></li>",
                    i = i,
                    title = ex.title,
                )
                .unwrap();
            }
        }
    }
    html.push_str("    </ul>\n");
    html.push_str("  </nav>\n");
}

fn write_header(html: &mut String) {
    html.push_str(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>docsrs Output Showcase</title>
  <style>
    :root {
      --bg: #1a1b26;
      --bg-surface: #24283b;
      --bg-sidebar: #16161e;
      --text: #a9b1d6;
      --text-muted: #565f89;
      --accent: #7aa2f7;
      --border: #292e42;
      --command-bg: #1f2335;
    }
    * { margin: 0; padding: 0; box-sizing: border-box; }
    html { scroll-behavior: smooth; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      display: flex;
      min-height: 100vh;
    }
    #sidebar {
      position: sticky;
      top: 0;
      height: 100vh;
      width: 260px;
      min-width: 260px;
      background: var(--bg-sidebar);
      border-right: 1px solid var(--border);
      overflow-y: auto;
      padding: 20px 0;
    }
    .sidebar-header {
      font-size: 18px;
      font-weight: 700;
      color: var(--accent);
      padding: 0 20px 16px;
      border-bottom: 1px solid var(--border);
      margin-bottom: 12px;
    }
    #sidebar ul { list-style: none; }
    #sidebar .cat-header {
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--text-muted);
      padding: 12px 20px 4px;
    }
    #sidebar a {
      display: block;
      padding: 6px 20px;
      color: var(--text);
      text-decoration: none;
      font-size: 13px;
      border-left: 3px solid transparent;
      transition: all 0.15s;
    }
    #sidebar a:hover {
      background: var(--bg-surface);
      color: var(--accent);
      border-left-color: var(--accent);
    }
    main {
      flex: 1;
      max-width: 960px;
      padding: 40px;
    }
    main > h1 {
      font-size: 28px;
      color: #c0caf5;
      margin-bottom: 8px;
    }
    .subtitle {
      color: var(--text-muted);
      margin-bottom: 40px;
      font-size: 14px;
    }
    section {
      margin-bottom: 48px;
    }
    section h2 {
      font-size: 18px;
      color: #c0caf5;
      margin-bottom: 12px;
      padding-bottom: 8px;
      border-bottom: 1px solid var(--border);
    }
    .category-tag {
      display: inline-block;
      font-size: 10px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      background: var(--accent);
      color: var(--bg);
      padding: 2px 8px;
      border-radius: 3px;
      vertical-align: middle;
      margin-right: 6px;
    }
    .command {
      font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace;
      font-size: 13px;
      background: var(--command-bg);
      color: var(--accent);
      padding: 10px 16px;
      border-radius: 6px 6px 0 0;
      border: 1px solid var(--border);
      border-bottom: none;
    }
    .error-tag {
      display: inline-block;
      font-size: 10px;
      font-weight: 600;
      background: #f7768e;
      color: var(--bg);
      padding: 1px 6px;
      border-radius: 3px;
      margin-left: 8px;
      vertical-align: middle;
    }
    .output {
      font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', Menlo, monospace;
      font-size: 13px;
      line-height: 1.5;
      background: var(--bg-surface);
      padding: 16px;
      border-radius: 0 0 6px 6px;
      border: 1px solid var(--border);
      overflow-x: auto;
      white-space: pre-wrap;
      word-wrap: break-word;
    }
    @media (max-width: 800px) {
      body { flex-direction: column; }
      #sidebar {
        position: static;
        width: 100%;
        height: auto;
        border-right: none;
        border-bottom: 1px solid var(--border);
      }
      main { padding: 20px; }
    }
  </style>
</head>
<body>
"#,
    );
}

fn write_footer(html: &mut String) {
    html.push_str(
        r#"</body>
</html>
"#,
    );
}
