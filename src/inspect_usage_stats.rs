use kodegen_mcp_schema::{Tool, ToolExecutionContext, ToolArgs, ToolResponse};
use kodegen_mcp_schema::McpError;
use kodegen_mcp_schema::introspection::{
    InspectUsageStatsArgs, InspectUsageOutput, InspectUsageStatsPrompts,
    ToolUsageStats, INSPECT_USAGE_STATS,
};
use kodegend_client_ipc::get_usage_stats;

// ============================================================================
// TOOL STRUCT
// ============================================================================

#[derive(Clone, Default)]
pub struct InspectUsageStatsTool;

impl InspectUsageStatsTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

// ============================================================================
// TOOL IMPLEMENTATION
// ============================================================================

impl Tool for InspectUsageStatsTool {
    type Args = InspectUsageStatsArgs;
    type Prompts = InspectUsageStatsPrompts;

    fn name() -> &'static str {
        INSPECT_USAGE_STATS
    }

    fn description() -> &'static str {
        "Get aggregated usage statistics for tool calls across all backend servers. \
         Returns metrics including total calls, success rates, per-tool statistics, \
         and session duration.\n\n\
         Useful for:\n\
         - Monitoring tool usage patterns\n\
         - Analyzing performance and success rates\n\
         - Debugging tool execution issues\n\
         - Understanding which tools are most frequently used\n\n\
         Note: Statistics are aggregated across all backend servers and include \
         both successful and failed calls."
    }

    fn read_only() -> bool {
        true
    }

    fn destructive() -> bool {
        false
    }

    fn idempotent() -> bool {
        true
    }

    fn open_world() -> bool {
        false
    }

    async fn execute(&self, _args: Self::Args, ctx: ToolExecutionContext) -> Result<ToolResponse<<Self::Args as ToolArgs>::Output>, McpError> {
        // Get connection ID from context
        let connection_id = ctx.connection_id()
            .ok_or_else(|| McpError::Other(anyhow::anyhow!("No connection ID available - usage stats require connection context")))?;

        // Query kodegend daemon via IPC for aggregated usage statistics
        let aggregated = get_usage_stats(connection_id)
            .map_err(|e| McpError::Other(anyhow::anyhow!("Failed to query usage stats from kodegend: {}", e)))?;

        // Aggregate statistics across all available servers
        let mut total_calls = 0u64;
        let mut successful_calls = 0u64;
        let mut failed_calls = 0u64;
        let mut tool_usage_map = std::collections::HashMap::new();

        for server in &aggregated.servers {
            // Only process servers that responded successfully
            if server.available {
                total_calls += server.stats.total_tool_calls;
                successful_calls += server.stats.successful_calls;
                failed_calls += server.stats.failed_calls;

                // Aggregate per-tool counts (tool_counts is a HashMap<String, u64>)
                for (tool_name, count) in &server.stats.tool_counts {
                    *tool_usage_map.entry(tool_name.clone()).or_insert(0u64) += count;
                }
            }
        }

        // Convert tool usage map to vector of ToolUsageStats
        // Note: We don't have duration data in the usage stats, only in history
        let tool_usage: Vec<ToolUsageStats> = tool_usage_map
            .into_iter()
            .map(|(tool_name, call_count)| {
                ToolUsageStats {
                    tool_name,
                    call_count: call_count as usize,
                    total_duration_ms: 0, // Duration tracking is in tool history, not usage stats
                    avg_duration_ms: 0,
                }
            })
            .collect();

        let success_rate = if total_calls > 0 {
            (successful_calls as f64 / total_calls as f64) * 100.0
        } else {
            0.0
        };

        // Calculate session duration from first_used to last_used across all servers
        let session_duration_ms = aggregated.servers
            .iter()
            .filter(|s| s.available)
            .map(|s| {
                let duration = s.stats.last_used.saturating_sub(s.stats.first_used);
                duration.max(0) as u64
            })
            .max()
            .unwrap_or(0);

        // Terminal formatted summary
        let summary = format!(
            "\x1b[35mUsage Statistics\x1b[0m\n\
             Total: {} · Success: {} · Failed: {} · Rate: {:.1}%",
            total_calls,
            successful_calls,
            failed_calls,
            success_rate
        );

        let output = InspectUsageOutput {
            success: true,
            total_calls: total_calls as usize,
            tools_used: tool_usage.len(),
            tool_usage,
            session_duration_ms,
            success_rate,
            successful_calls: successful_calls as usize,
            failed_calls: failed_calls as usize,
        };

        Ok(ToolResponse::new(summary, output))
    }
}
