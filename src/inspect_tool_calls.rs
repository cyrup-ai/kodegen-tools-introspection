use kodegen_mcp_tool::{Tool, ToolExecutionContext, ToolArgs, ToolResponse};
use kodegen_mcp_tool::error::McpError;
use kodegen_mcp_tool::tool_history;
use kodegen_mcp_schema::introspection::{InspectToolCallsArgs, InspectToolCallsPromptArgs, InspectToolCallsOutput, ToolCallRecord, INSPECT_TOOL_CALLS};
use rmcp::model::{PromptArgument, PromptMessage, PromptMessageContent, PromptMessageRole};

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
    type PromptArgs = InspectToolCallsPromptArgs;

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

        // Convert calls to typed output format (serialize Value to JSON strings)
        let typed_calls: Vec<ToolCallRecord> = calls.iter().map(|c| ToolCallRecord {
            tool_name: c.tool_name.clone(),
            timestamp: c.timestamp.clone(),
            duration_ms: c.duration_ms,
            args_json: serde_json::to_string(&c.arguments).unwrap_or_default(),
            output_json: serde_json::to_string(&c.output).unwrap_or_default(),
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

    fn prompt_arguments() -> Vec<PromptArgument> {
        vec![]
    }

    async fn prompt(&self, _args: Self::PromptArgs) -> Result<Vec<PromptMessage>, McpError> {
        Ok(vec![
            PromptMessage {
                role: PromptMessageRole::User,
                content: PromptMessageContent::text(
                    "How do I use inspect_tool_calls to see what work has been done?",
                ),
            },
            PromptMessage {
                role: PromptMessageRole::Assistant,
                content: PromptMessageContent::text(
                    "The inspect_tool_calls tool helps you understand what tools have been \
                     executed and what they did. This is especially useful when:\n\n\
                     1. **New chat context**: You join a new chat and want to understand what \
                     work was already done\n\n\
                     2. **Debugging**: You want to trace the sequence of operations that led \
                     to the current state\n\n\
                     3. **Learning**: You want to see how tools were used together to accomplish \
                     a task\n\n\
                     Usage examples:\n\n\
                     ```\n\
                     # Get first 50 tool calls (default)\n\
                     inspect_tool_calls({})\n\n\
                     # Get first 100 calls\n\
                     inspect_tool_calls({ max_results: 100 })\n\n\
                     # Get calls 50-99 (pagination)\n\
                     inspect_tool_calls({ offset: 50, max_results: 50 })\n\n\
                     # Get last 20 calls (most recent)\n\
                     inspect_tool_calls({ offset: -20 })\n\n\
                     # Get last 10 read_file calls\n\
                     inspect_tool_calls({ tool_name: \"read_file\", offset: -10 })\n\n\
                     # Get only read_file calls\n\
                     inspect_tool_calls({ tool_name: \"read_file\" })\n\n\
                     # Get calls since a specific timestamp\n\
                     inspect_tool_calls({ since: \"2024-10-12T20:00:00Z\" })\n\
                     ```\n\n\
                     The response includes:\n\
                     - Timestamp of each call\n\
                     - Tool name\n\
                     - Arguments passed\n\
                     - Output received\n\
                     - Execution duration in milliseconds\n\n\
                     Note: History is kept in memory (last 1000 calls) and persisted to \
                     ~/.config/kodegen-mcp/tool-history.jsonl for durability across restarts.",
                ),
            },
        ])
    }
}
