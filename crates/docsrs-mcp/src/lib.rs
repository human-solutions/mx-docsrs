use rmcp::handler::server::tool::{ToolCallContext, ToolRouter};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, Implementation, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Clone)]
pub struct DocsRsServer {
    tool_router: ToolRouter<Self>,
}

impl Default for DocsRsServer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct LookupDocsParams {
    /// Crate path: crate[@version][::path]. Hyphens are normalized to underscores. Examples: "tokio", "serde@1.0", "tokio::task::spawn"
    pub crate_spec: String,
    /// Text filter (substring match). Single match returns full docs; multiple returns a sorted list.
    #[serde(default)]
    pub filter: Option<String>,
}

#[tool_router]
impl DocsRsServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Fetch Rust documentation from docs.rs or local workspace crates.

Modes:
- Crate root: \"serde\" → crate docs + public items
- Path lookup: \"serde::Deserialize\" → full docs for that item
- Search: \"serde\", filter: \"Map\" → list matching items (or full docs if exactly one match)

Version resolution (no @version):
- Dependency in Cargo.toml: locked version
- Local workspace crate: builds locally
- Otherwise: fetches latest from docs.rs

Examples:
- crate_spec: \"serde@1.0\" → pinned
- crate_spec: \"tokio::task\", filter: \"spawn\" → scoped search"
    )]
    async fn lookup_docs(
        &self,
        params: Parameters<LookupDocsParams>,
    ) -> Result<CallToolResult, McpError> {
        let crate_spec = params.0.crate_spec;
        let filter = params.0.filter;

        let result = tokio::task::spawn_blocking(move || {
            if let Some(filter) = filter {
                docsrs_core::run_cli(&[&crate_spec, &filter])
            } else {
                docsrs_core::run_cli(&[&crate_spec])
            }
        })
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

        match result {
            Ok(docs) => Ok(CallToolResult::success(vec![Content::text(docs)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
        }
    }
}

impl ServerHandler for DocsRsServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: Implementation {
                name: "docsrs".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context).await
    }
}
