use docsrs_mcp::DocsRsServer;
use rmcp::ClientHandler;
use rmcp::model::{CallToolRequestParam, ClientInfo, Implementation};
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
    let (client_io, server_io) = tokio::io::duplex(4096);

    // Start server in background
    let server_handle = tokio::spawn(async move {
        let _ = server.serve(server_io).await.unwrap().waiting().await;
    });

    // Connect client
    let client_service = client.serve(client_io).await.unwrap();

    // Call the tool
    let request = CallToolRequestParam {
        name: tool.into(),
        arguments: Some(args.as_object().cloned().unwrap_or_default()),
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
    // Same normalization as CLI tests
    output
        .lines()
        .map(|line| {
            if line.starts_with("Local crate found at: ") {
                "Local crate found at: [LOCAL_PATH]".to_string()
            } else if let Some(rest) = line.strip_prefix("Using local dependency at /") {
                if let Some(version_start) = rest.find(" (version ") {
                    format!(
                        "Using local dependency at [LOCAL_PATH]{}",
                        &rest[version_start..]
                    )
                } else {
                    line.to_string()
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    Using dependency serde@1.0.228
    pub use serde::Deserialize

    Derive macro available if serde is built with `features = ["derive"]`.
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
    insta::assert_snapshot!(output, @"http status: 404");
}
