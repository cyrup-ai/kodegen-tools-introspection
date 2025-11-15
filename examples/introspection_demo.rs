mod common;

use anyhow::Context;
use kodegen_mcp_schema::introspection::*;
use serde_json::json;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info").init();

    info!("Starting introspection tools example");

    // Connect to kodegen server with introspection category
    let (conn, mut server) =
        common::connect_to_local_http_server().await?;

    // Wrap client with logging
    let workspace_root = common::find_workspace_root()
        .context("Failed to find workspace root")?;
    let log_path = workspace_root.join("tmp/mcp-client/introspection.log");
    let client = common::LoggingClient::new(conn.client(), log_path)
        .await
        .context("Failed to create logging client")?;

    info!("Connected to server: {:?}", client.server_info());

    // 1. INSPECT_USAGE_STATS - Get usage statistics
    info!("1. Testing inspect_usage_stats");
    match client.call_tool(INSPECT_USAGE_STATS, json!({})).await {
        Ok(result) => info!("Usage stats: {:?}", result),
        Err(e) => error!("Failed to get usage stats: {}", e),
    }

    // 2. INSPECT_TOOL_CALLS - Get recent tool call history
    info!("2. Testing inspect_tool_calls");
    match client
        .call_tool(INSPECT_TOOL_CALLS, json!({ "max_results": 10 }))
        .await
    {
        Ok(result) => info!("Recent tool calls: {:?}", result),
        Err(e) => error!("Failed to get recent tool calls: {}", e),
    }

    // Graceful shutdown
    conn.close().await?;
    server.shutdown().await?;
    info!("Introspection tools example completed successfully");

    Ok(())
}
