//! Introspection tools for monitoring and debugging tool usage
//!
//! This module provides tools for understanding how tools are being used,
//! viewing execution history, and analyzing usage patterns.

mod inspect_tool_calls;
mod inspect_usage_stats;

pub use inspect_tool_calls::InspectToolCallsTool;
pub use inspect_usage_stats::InspectUsageStatsTool;

/// Start the introspection HTTP server programmatically
///
/// Returns a ServerHandle for graceful shutdown control.
/// This function is non-blocking - the server runs in background tasks.
///
/// # Arguments
/// * `addr` - Socket address to bind to (e.g., "127.0.0.1:30447")
/// * `tls_cert` - Optional path to TLS certificate file
/// * `tls_key` - Optional path to TLS private key file
///
/// # Returns
/// ServerHandle for graceful shutdown, or error if startup fails
pub async fn start_server(
    addr: std::net::SocketAddr,
    tls_cert: Option<std::path::PathBuf>,
    tls_key: Option<std::path::PathBuf>,
) -> anyhow::Result<kodegen_server_http::ServerHandle> {
    use kodegen_server_http::{create_http_server, Managers, RouterSet, register_tool};
    use rmcp::handler::server::router::{prompt::PromptRouter, tool::ToolRouter};
    use std::time::Duration;

    let tls_config = match (tls_cert, tls_key) {
        (Some(cert), Some(key)) => Some((cert, key)),
        _ => None,
    };

    let shutdown_timeout = Duration::from_secs(30);
    let session_keep_alive = Duration::from_secs(300); // 5 minutes

    create_http_server("introspection", addr, tls_config, shutdown_timeout, session_keep_alive, |_config, tracker| {
        let usage_tracker = tracker.clone();
        Box::pin(async move {
            let mut tool_router = ToolRouter::new();
            let mut prompt_router = PromptRouter::new();
            let managers = Managers::new();

            // Register all 2 introspection tools
            (tool_router, prompt_router) = register_tool(
                tool_router,
                prompt_router,
                crate::InspectUsageStatsTool::new(usage_tracker.clone()),
            );

            (tool_router, prompt_router) = register_tool(
                tool_router,
                prompt_router,
                crate::InspectToolCallsTool::new(),
            );

            Ok(RouterSet::new(tool_router, prompt_router, managers))
        })
    }).await
}
