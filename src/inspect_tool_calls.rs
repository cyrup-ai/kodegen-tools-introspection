use kodegen_mcp_schema::{Tool, ToolExecutionContext, ToolArgs, ToolResponse};
use kodegen_mcp_schema::McpError;
use kodegen_mcp_schema::introspection::{InspectToolCallsArgs, InspectToolCallsOutput, InspectToolCallsPrompts, ToolCallRecord, INSPECT_TOOL_CALLS};
use kodegend_client_ipc::get_tool_history;


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

    async fn execute(&self, args: Self::Args, ctx: ToolExecutionContext) -> Result<ToolResponse<<Self::Args as ToolArgs>::Output>, McpError> {
        // Get connection ID from context
        let connection_id = ctx.connection_id()
            .ok_or_else(|| McpError::Other(anyhow::anyhow!("No connection ID available - tool history requires connection context")))?;

        // Query kodegend daemon via IPC for aggregated tool history
        let history = get_tool_history(connection_id)
            .map_err(|e| McpError::Other(anyhow::anyhow!("Failed to query tool history from kodegend: {}", e)))?;

        // Flatten all calls from all servers and map IPC types to schema types
        let mut all_calls: Vec<ToolCallRecord> = history.servers
            .into_iter()
            .flat_map(|server| server.calls)
            .map(|ipc_call| ToolCallRecord {
                tool_name: ipc_call.tool_name,
                timestamp: ipc_call.timestamp,
                duration_ms: ipc_call.duration_ms,
                args_json: ipc_call.args_json,
                output_json: ipc_call.output_json,
            })
            .collect();

        // Apply tool name filter
        if let Some(ref tool_name) = args.tool_name {
            all_calls.retain(|call| &call.tool_name == tool_name);
        }

        // Apply timestamp filter (since)
        if let Some(ref since) = args.since {
            all_calls.retain(|call| call.timestamp >= *since);
        }

        // Sort by timestamp descending (newest first)
        all_calls.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply offset and limit
        let offset = args.offset;
        let max_results = args.max_results;

        let start_idx = if offset < 0 {
            // Negative offset = tail behavior (last N items)
            let abs_offset: usize = offset.unsigned_abs().try_into().unwrap();
            all_calls.len().saturating_sub(abs_offset)
        } else {
            offset as usize
        };

        let calls: Vec<ToolCallRecord> = all_calls
            .into_iter()
            .skip(start_idx)
            .take(max_results)
            .collect();

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

        let output = InspectToolCallsOutput {
            success: true,
            count: calls.len(),
            total_entries_in_memory: history.total_calls,
            calls,
            filter_tool_name: args.tool_name,
            filter_since: args.since,
            offset: args.offset,
            max_results: args.max_results,
        };

        Ok(ToolResponse::new(summary, output))
    }

}
