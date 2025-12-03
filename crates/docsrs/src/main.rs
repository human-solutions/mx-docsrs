use std::process;

use docsrs_mcp::DocsRsServer;
use rmcp::service::ServiceExt;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Check for --mcp flag (ignores all other args)
    if args.iter().any(|a| a == "--mcp") {
        run_mcp_server().await;
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

async fn run_mcp_server() {
    let server = DocsRsServer::new();
    let transport = rmcp::transport::stdio();
    match server.serve(transport).await {
        Ok(running) => {
            if let Err(e) = running.waiting().await {
                eprintln!("MCP server error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("MCP server failed to start: {}", e);
            process::exit(1);
        }
    }
}
