//! Introspection tools for monitoring and debugging tool usage
//!
//! This module provides tools for understanding how tools are being used,
//! viewing execution history, and analyzing usage patterns.

mod get_recent_tool_calls;
mod get_usage_stats;

pub use get_recent_tool_calls::GetRecentToolCallsTool;
pub use get_usage_stats::GetUsageStatsTool;

/// Start the introspection HTTP server programmatically for embedded mode
pub async fn start_server(
    addr: std::net::SocketAddr,
    tls_cert: Option<std::path::PathBuf>,
    tls_key: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    use kodegen_server_http::{Managers, RouterSet, register_tool};
    use kodegen_tools_config::ConfigManager;
    use rmcp::handler::server::router::{prompt::PromptRouter, tool::ToolRouter};
    use std::sync::Arc;

    // Initialize logging (idempotent)
    let _ = env_logger::try_init();

    // Install rustls provider (idempotent)
    if rustls::crypto::ring::default_provider().install_default().is_err() {
        log::debug!("rustls crypto provider already installed");
    }

    // Initialize config
    let config = ConfigManager::new();
    config.init().await?;

    // Create instance ID and usage tracker
    let timestamp = chrono::Utc::now();
    let pid = std::process::id();
    let instance_id = format!("{}-{}", timestamp.format("%Y%m%d-%H%M%S-%9f"), pid);
    let usage_tracker = kodegen_utils::usage_tracker::UsageTracker::new(
        format!("introspection-{}", instance_id)
    );

    // Initialize global tool history
    kodegen_mcp_tool::tool_history::init_global_history(instance_id.clone()).await;

    // Create routers
    let mut tool_router = ToolRouter::new();
    let mut prompt_router = PromptRouter::new();
    let managers = Managers::new();

    // Register all 2 introspection tools
    (tool_router, prompt_router) = register_tool(
        tool_router,
        prompt_router,
        crate::GetUsageStatsTool::new(usage_tracker.clone()),
    );

    (tool_router, prompt_router) = register_tool(
        tool_router,
        prompt_router,
        crate::GetRecentToolCallsTool::new(),
    );

    // Create router set
    let router_set = RouterSet::new(tool_router, prompt_router, managers);

    // Create session manager
    let session_config = rmcp::transport::streamable_http_server::session::local::SessionConfig {
        channel_capacity: 16,
        keep_alive: Some(std::time::Duration::from_secs(3600)),
    };
    let session_manager = Arc::new(
        rmcp::transport::streamable_http_server::session::local::LocalSessionManager {
            sessions: Default::default(),
            session_config,
        }
    );

    // Create HTTP server
    let server = kodegen_server_http::HttpServer::new(
        router_set.tool_router,
        router_set.prompt_router,
        usage_tracker,
        config,
        router_set.managers,
        session_manager,
    );

    // Start server
    let shutdown_timeout = std::time::Duration::from_secs(30);
    let tls_config = tls_cert.zip(tls_key);
    let handle = server.serve_with_tls(addr, tls_config, shutdown_timeout).await?;

    // Wait for completion (kodegend controls shutdown)
    handle.wait_for_completion(shutdown_timeout).await
        .map_err(|e| anyhow::anyhow!("Server shutdown error: {}", e))?;

    Ok(())
}
