use kodegen_mcp_schema::{Tool, ToolExecutionContext, ToolArgs, ToolResponse};
use kodegen_mcp_schema::McpError;
use kodegen_mcp_schema::introspection::{InspectUsageStatsArgs, InspectUsageStatsPrompts, InspectUsageOutput, ToolUsageStats, INSPECT_USAGE_STATS};
use kodegen_utils::usage_tracker::UsageTracker;


// ============================================================================
// TOOL STRUCT
// ============================================================================

#[derive(Clone)]
pub struct InspectUsageStatsTool {
    usage_tracker: UsageTracker,
}

impl InspectUsageStatsTool {
    #[must_use]
    pub fn new(usage_tracker: UsageTracker) -> Self {
        Self { usage_tracker }
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
        "Get usage statistics for debugging and analysis. Returns summary of tool usage, \
         success/failure rates, and performance metrics."
    }

    async fn execute(&self, _args: Self::Args, _ctx: ToolExecutionContext) -> Result<ToolResponse<<Self::Args as ToolArgs>::Output>, McpError> {
        // Terminal formatted summary (compact 2-line format)
        let summary = self.usage_tracker.get_formatted_summary();

        // Get stats from usage tracker
        let stats = self.usage_tracker.get_stats();

        // Convert per-tool stats to typed format
        let tool_usage: Vec<ToolUsageStats> = stats.tool_counts
            .iter()
            .map(|(name, count): (&String, &u64)| ToolUsageStats {
                tool_name: name.clone(),
                call_count: *count as usize,
                total_duration_ms: 0, // Not tracked in current implementation
                avg_duration_ms: 0,   // Not tracked in current implementation
            })
            .collect();

        // Calculate session duration from timestamps
        let session_duration_ms = if stats.last_used > stats.first_used {
            ((stats.last_used - stats.first_used) * 1000) as u64
        } else {
            0
        };

        // Calculate success rate
        let success_rate = if stats.total_tool_calls > 0 {
            (stats.successful_calls as f64 / stats.total_tool_calls as f64) * 100.0
        } else {
            100.0
        };

        let output = InspectUsageOutput {
            success: true,
            total_calls: stats.total_tool_calls as usize,
            tools_used: stats.tool_counts.len(),
            tool_usage,
            session_duration_ms,
            success_rate,
            successful_calls: stats.successful_calls as usize,
            failed_calls: stats.failed_calls as usize,
        };

        Ok(ToolResponse::new(summary, output))
    }

    // Behavior annotations
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
}
