use rmcp::handler::server::tool::{ToolCallContext, ToolRouter};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
    PaginatedRequestParam, ServerCapabilities, ServerInfo,
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
    /// Crate path: crate[@version][::path] (e.g., "tokio", "serde@1.0", "tokio::task::spawn")
    pub crate_spec: String,
    /// Optional text filter to search for (e.g., "spawn", "async"). When provided, returns matching items instead of full documentation.
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
        description = "Look up Rust documentation for a crate, module, or item. Without a filter, returns full documentation. With a filter, searches for matching items."
    )]
    async fn lookup_docs(
        &self,
        params: Parameters<LookupDocsParams>,
    ) -> Result<CallToolResult, McpError> {
        let crate_spec = params.0.crate_spec;
        let filter = params.0.filter;

        let result = tokio::task::spawn_blocking(move || {
            if let Some(filter) = filter {
                mx_docsrs::run_cli(&[&crate_spec, &filter])
            } else {
                mx_docsrs::run_cli(&[&crate_spec])
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
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_context).await
    }
}
