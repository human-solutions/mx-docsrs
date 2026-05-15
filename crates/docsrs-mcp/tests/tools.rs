use docsrs_mcp::DocsRsServer;
use rmcp::ClientHandler;
use rmcp::model::{CallToolRequestParams, ClientInfo, Implementation};
use rmcp::service::ServiceExt;

/// Minimal client handler for testing
#[derive(Clone)]
struct TestClient;

impl ClientHandler for TestClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo {
            client_info: Implementation {
                name: "test-client".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

async fn call_tool(tool: impl Into<String>, args: serde_json::Value) -> (String, bool) {
    let tool: String = tool.into();
    // Disable colors for consistent test output
    colored::control::set_override(false);

    let server = DocsRsServer::new();
    let client = TestClient;

    // Create bidirectional in-memory transport using duplex streams
    let (client_io, server_io) = tokio::io::duplex(1024 * 1024);

    // Start server in background
    let server_handle = tokio::spawn(async move {
        let _ = server.serve(server_io).await.unwrap().waiting().await;
    });

    // Connect client
    let client_service = client.serve(client_io).await.unwrap();

    // Call the tool
    let request = CallToolRequestParams {
        name: tool.into(),
        arguments: Some(args.as_object().cloned().unwrap_or_default()),
        meta: None,
        task: None,
    };

    let result = client_service.call_tool(request).await.unwrap();

    // Extract text content and error status
    let is_error = result.is_error.unwrap_or(false);
    let output = result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n");

    // Clean up
    drop(client_service);
    server_handle.abort();

    (normalize_output(&output), is_error)
}

fn normalize_output(output: &str) -> String {
    // No machine-specific paths in the new comment format
    output.to_string()
}

#[tokio::test]
async fn lookup_docs_crate() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "serde"
        }),
    )
    .await;
    assert!(!is_error, "lookup_docs should not fail for valid crate");
    insta::assert_snapshot!(output);
}

#[tokio::test]
async fn lookup_docs_with_path() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "serde::Deserialize"
        }),
    )
    .await;
    assert!(
        !is_error,
        "lookup_docs should not fail for valid crate path"
    );
    insta::assert_snapshot!(output, @r#"
    // dependency serde@1.0.228
    // found serde::Deserialize

    /// Derive macro available if serde is built with `features = ["derive"]`.
    pub use serde::Deserialize
    "#);
}

#[tokio::test]
async fn lookup_docs_with_filter() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "serde",
            "filter": "Deserialize"
        }),
    )
    .await;
    assert!(!is_error, "lookup_docs with filter should not fail");
    insta::assert_snapshot!(output);
}

#[tokio::test]
async fn lookup_invalid_crate() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "nonexistent_crate_12345"
        }),
    )
    .await;
    assert!(is_error, "lookup_docs should fail for invalid crate");
    insta::assert_snapshot!(output, @"Crate 'nonexistent_crate_12345@latest' not found on docs.rs. Check the crate name and version.");
}

// --- Additional end-to-end MCP tests against external crates ---
//
// These pin specific versions of well-known crates that are confirmed to
// fetch from docs.rs and parse with the workspace's rustdoc-types version.

#[tokio::test]
async fn lookup_struct_via_path() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "anyhow@1.0.99::Error"
        }),
    )
    .await;
    assert!(!is_error, "expected success; got error:\n{output}");
    assert!(
        output.starts_with("// found struct anyhow::Error"),
        "expected struct description; got:\n{output}"
    );
    assert!(output.contains("pub struct anyhow::Error"));
}

#[tokio::test]
async fn lookup_with_exact_suffix_filter() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "anyhow@1.0.99",
            "filter": "Error"
        }),
    )
    .await;
    assert!(!is_error, "expected success; got error:\n{output}");
    assert!(
        output.starts_with("// found struct anyhow::Error"),
        "expected single-match description; got:\n{output}"
    );
}

#[tokio::test]
async fn lookup_with_path_scoped_filter() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "tokio@1.40.5::sync::mpsc",
            "filter": "channel"
        }),
    )
    .await;
    assert!(!is_error, "expected success; got error:\n{output}");
    assert!(
        output.starts_with("// 2 items matching \"channel\""),
        "expected 2-item list description; got:\n{output}"
    );
    assert!(output.contains("fn tokio::sync::mpsc::channel"));
    assert!(output.contains("fn tokio::sync::mpsc::unbounded_channel"));
}

#[tokio::test]
async fn lookup_type_alias_kind() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "anyhow@1.0.99::Result"
        }),
    )
    .await;
    assert!(!is_error, "expected success; got error:\n{output}");
    assert!(
        output.starts_with("// found type anyhow::Result"),
        "expected type-alias description; got:\n{output}"
    );
}

#[tokio::test]
async fn lookup_unknown_version_reports_error() {
    let (output, is_error) = call_tool(
        "lookup_docs",
        serde_json::json!({
            "crate_spec": "anyhow@99.99.99"
        }),
    )
    .await;
    assert!(is_error, "expected error for unknown version");
    assert!(
        output.contains("not found on docs.rs"),
        "expected not-found message; got:\n{output}"
    );
}
