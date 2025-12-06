use kodegen_mcp_schema::{Tool, ToolExecutionContext, ToolArgs, ToolResponse};
use kodegen_mcp_schema::McpError;
use kodegen_mcp_schema::tool::tool_history;
use kodegen_mcp_schema::introspection::{InspectToolCallsArgs, InspectToolCallsOutput, InspectToolCallsPrompts, ToolCallRecord, INSPECT_TOOL_CALLS};


// ============================================================================
// TOOL STRUCT
// ============================================================================

#[derive(Clone, Default)]
pub struct InspectToolCallsTool;

impl InspectToolCallsTool {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

// ============================================================================
// TOOL IMPLEMENTATION
// ============================================================================

impl Tool for InspectToolCallsTool {
    type Args = InspectToolCallsArgs;
    type Prompts = InspectToolCallsPrompts;

    fn name() -> &'static str {
        INSPECT_TOOL_CALLS
    }

    fn description() -> &'static str {
        "Get recent tool call history with their arguments and outputs. \
         Returns chronological list of tool calls made during this session. \
         Supports pagination via offset parameter (negative for tail behavior).\n\n\
         Useful for:\n\
         - Onboarding new chats about work already done\n\
         - Recovering context after chat history loss\n\
         - Debugging tool call sequences\n\
         - Navigating large tool histories with pagination\n\n\
         Note: Does not track its own calls or other meta/query tools. \
         History kept in memory (last 1000 calls, persisted to disk)."
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

    async fn execute(&self, args: Self::Args, _ctx: ToolExecutionContext) -> Result<ToolResponse<<Self::Args as ToolArgs>::Output>, McpError> {
        let history = tool_history::get_global_history()
            .ok_or_else(|| McpError::Other(anyhow::anyhow!("Tool history not initialized")))?;

        let calls = history
            .get_recent_calls(
                args.max_results,
                args.offset,
                args.tool_name.as_deref(),
                args.since.as_deref(),
            )
            .await;

        let stats = history.get_stats().await;

        // Terminal formatted summary
        let summary = if calls.is_empty() {
            "\x1b[35m󰋚 Tool Call History\x1b[0m\n\
             󰘖 Calls: 0 · No calls matching criteria".to_string()
        } else {
            let latest_tool = calls.first()
                .map(|c| c.tool_name.as_str())
                .unwrap_or("unknown");

            format!(
                "\x1b[35m󰋚 Tool Call History\x1b[0m\n\
                 󰘖 Calls: {} · Latest: {}",
                calls.len(),
                latest_tool
            )
        };

        // Convert calls to typed output format (direct field mapping)
        let typed_calls: Vec<ToolCallRecord> = calls.iter().map(|c| ToolCallRecord {
            tool_name: c.tool_name.clone(),
            timestamp: c.timestamp.clone(),
            duration_ms: c.duration_ms,
            args_json: c.args_json.clone(),
            output_json: c.output_json.clone(),
        }).collect();

        let output = InspectToolCallsOutput {
            success: true,
            count: typed_calls.len(),
            total_entries_in_memory: stats.total_entries,
            calls: typed_calls,
            filter_tool_name: args.tool_name,
            filter_since: args.since,
            offset: args.offset,
            max_results: args.max_results,
        };

        Ok(ToolResponse::new(summary, output))
    }

}
