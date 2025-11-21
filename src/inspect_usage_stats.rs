use kodegen_mcp_tool::{Tool, ToolExecutionContext};
use kodegen_mcp_tool::error::McpError;
use kodegen_mcp_schema::introspection::{InspectUsageStatsArgs, InspectUsageStatsPromptArgs, INSPECT_USAGE_STATS};
use kodegen_utils::usage_tracker::UsageTracker;
use rmcp::model::{Content, PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole};

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

    async fn execute(&self, _args: Self::Args, _ctx: ToolExecutionContext) -> Result<Vec<Content>, McpError> {
        let mut contents = Vec::new();

        // Content 1: Terminal formatted summary (compact 2-line format)
        let summary = self.usage_tracker.get_formatted_summary();
        contents.push(Content::text(summary));

        // Content 2: JSON metadata (use existing get_stats method)
        let stats = self.usage_tracker.get_stats();
        let json_str = serde_json::to_string_pretty(&stats)
            .unwrap_or_else(|_| "{}".to_string());
        contents.push(Content::text(json_str));

        Ok(contents)
    }

    fn prompt_arguments() -> Vec<PromptArgument> {
        vec![]
    }

    async fn prompt(&self, _args: Self::PromptArgs) -> Result<Vec<PromptMessage>, McpError> {
        Ok(vec![
            PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text(
                    "How can I check what tools have been used and see usage statistics?",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::Assistant,
                content: PromptMessageContent::text(
                    "I can show you usage statistics using the inspect_usage_stats tool. \
                     This provides:\n\n\
                     - Total tool calls (successful and failed)\n\
                     - Success/failure rates\n\
                     - Breakdown by category (filesystem, terminal, edit, search, etc.)\n\
                     - Per-tool call counts\n\
                     - Session information\n\
                     - Timestamps\n\n\
                     Let me get those stats for you.",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text("Yes, please show me the stats."),
            },
            PromptMessage {
                role: PromptMessageRole::Assistant,
                content: PromptMessageContent::text(
                    "[Calling inspect_usage_stats tool]\n\n\
                     The statistics show all tool usage since the server started. \
                     This is useful for debugging, understanding usage patterns, \
                     and identifying frequently used tools.",
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
