use std::process;

use docsrs_mcp::DocsRsServer;
use rmcp::service::ServiceExt;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Check for --mcp flag (ignores all other args)
    if args.iter().any(|a| a == "--mcp") {
        run_mcp_server();
    } else {
        run_cli(&args);
    }
}

fn run_cli(args: &[String]) {
    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    match docsrs_core::run_cli(&args_refs) {
        Ok(output) => {
            print!("{}", output);
            process::exit(0);
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            process::exit(1);
        }
    }
}

#[tokio::main]
async fn run_mcp_server() {
    let server = DocsRsServer::new();
    let transport = rmcp::transport::stdio();
    if let Err(e) = server.serve(transport).await.unwrap().waiting().await {
        eprintln!("MCP server error: {}", e);
        process::exit(1);
    }
}
