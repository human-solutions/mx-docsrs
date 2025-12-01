use docsrs_mcp::DocsRsServer;
use rmcp::service::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = DocsRsServer::new();
    let transport = rmcp::transport::stdio();
    server.serve(transport).await?.waiting().await?;
    Ok(())
}
