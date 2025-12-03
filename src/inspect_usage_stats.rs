use kodegen_mcp_tool::{Tool, ToolExecutionContext, ToolArgs, ToolResponse};
use kodegen_mcp_tool::error::McpError;
use kodegen_mcp_schema::introspection::{InspectUsageStatsArgs, InspectUsageStatsPromptArgs, InspectUsageOutput, ToolUsageStats, INSPECT_USAGE_STATS};
use kodegen_utils::usage_tracker::UsageTracker;
use rmcp::model::{PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole};

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
    type PromptArgs = InspectUsageStatsPromptArgs;

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

    fn prompt_arguments() -> Vec<PromptArgument> {
        vec![]
    }

    async fn prompt(&self, _args: Self::PromptArgs) -> Result<Vec<PromptMessage>, McpError> {
        Ok(vec![
            PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text(
                    "How can I track tool usage and debug failures in my workflow? \
                     I want to understand which tools are being used and if any are failing.",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::Assistant,
                content: PromptMessageContent::text(
                    "The inspect_usage_stats tool is perfect for this. It provides comprehensive \
                     usage tracking and debugging insights:\n\n\
                     **When to use it:**\n\
                     - Mid-session debugging: When workflows seem slow or unreliable\n\
                     - End-of-session review: Understand what worked and what didn't\n\
                     - Performance analysis: Identify which tool categories are most used\n\
                     - Failure investigation: Spot tools with high failure rates\n\n\
                     **What it provides:**\n\
                     - Total tool calls with success/failure breakdown\n\
                     - Success rate percentages\n\
                     - Category-level statistics (filesystem, terminal, search, etc.)\n\
                     - Per-tool call counts\n\
                     - Session info and timestamps",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text(
                    "What do the statistics mean? How do I interpret the output?",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::Assistant,
                content: PromptMessageContent::text(
                    "The tool returns two formats:\n\n\
                     **1. Summary Line (compact format):**\n\
                     Example: \"Total: 42 calls, Success rate: 95.2% (40 successful, 2 failed)\"\n\n\
                     **2. JSON Statistics (detailed breakdown):**\n\
                     - `success_rate`: Calculated as (successful_calls / total_calls) × 100\n\
                     - `category_breakdown`: Shows distribution across tool types\n  \
                       Example: {\"filesystem\": 120, \"terminal\": 45, \"edit\": 30}\n\
                     - `per_tool_counts`: Individual tool usage\n  \
                       Example: {\"fs_read_file\": 80, \"fs_search\": 40}\n\
                     - `session_info`: When tracking started and current session ID\n\n\
                     **Key metrics to watch:**\n\
                     - Success rate above 90% is healthy\n\
                     - Category breakdown helps identify workflow patterns\n\
                     - Per-tool counts show which operations are most common",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text(
                    "How do I identify problematic tools that might be causing issues?",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::Assistant,
                content: PromptMessageContent::text(
                    "Look for these warning signs in the statistics:\n\n\
                     **Low success rates (< 80%):**\n\
                     - Indicates repeated failures with specific tools\n\
                     - May point to configuration issues or invalid inputs\n\n\
                     **High failure counts:**\n\
                     - Check per-tool breakdown for tools with many failed calls\n\
                     - Example: If fs_read_file shows 20 failures, files may not exist\n\n\
                     **Patterns to investigate:**\n\
                     - Consistent failures in one category (e.g., all terminal tools failing)\n\
                     - Sudden drop in success rate during session\n\
                     - Tools that work once then fail repeatedly\n\n\
                     **Important gotcha:**\n\
                     Failures don't always mean the tool is broken. Common causes:\n\
                     - Input validation errors (wrong file paths, invalid arguments)\n\
                     - Expected failures (checking if file exists, it doesn't)\n\
                     - Permission issues or missing dependencies\n\n\
                     Use the failure information to guide debugging, not as definitive proof of bugs.",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text(
                    "What's the difference between session stats and global stats? \
                     When should I use each?",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::Assistant,
                content: PromptMessageContent::text(
                    "The inspect_usage_stats tool tracks statistics for the current session:\n\n\
                     **Current Session Stats:**\n\
                     - Covers from when the MCP server started until now\n\
                     - Resets when server restarts\n\
                     - Best for: Active debugging of current workflow problems\n\
                     - Use when: Something seems wrong in your current work\n\n\
                     **Global/Historical Stats:**\n\
                     - Would require persistent storage (not currently implemented)\n\
                     - Would show long-term patterns across multiple sessions\n\
                     - Best for: Understanding typical usage over time\n\n\
                     **Best practices:**\n\
                     - Check stats mid-session if workflows slow down unexpectedly\n\
                     - Review at end of session to understand what was accomplished\n\
                     - Compare success rates across different types of tasks\n\
                     - Use category breakdown to optimize tool selection\n\n\
                     The session-scoped stats are perfect for immediate debugging and understanding \
                     current workflow health.",
                ),
            },
        ])
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
