# kodegen-tools-introspection

> Memory-efficient, Blazing-Fast MCP tools for code generation agents

Introspection tools for monitoring and debugging tool usage in AI agent systems. Part of the [KODEGEN.ᴀɪ](https://kodegen.ai) ecosystem.

## Features

This MCP server provides two essential introspection tools:

### 🔍 inspect_tool_calls

Get chronological tool call history with full arguments and outputs.

**Use cases:**
- Onboard new chat sessions by reviewing completed work
- Recover context after chat history loss
- Debug tool call sequences and execution flow
- Navigate large tool histories with flexible pagination

**Pagination support:**
```rust
// First 50 calls (default)
{ "max_results": 50 }

// Last 20 calls (tail behavior)
{ "offset": -20 }

// Calls 50-99 (range)
{ "offset": 50, "max_results": 50 }

// Filter by tool name
{ "tool_name": "read_file", "offset": -10 }

// Filter by timestamp
{ "since": "2024-10-12T20:00:00Z" }
```

### 📊 inspect_usage_stats

Get comprehensive usage statistics and performance metrics.

**Returns:**
- Total tool calls (successful and failed)
- Success/failure rates
- Breakdown by category (filesystem, terminal, edit, search, etc.)
- Per-tool call counts
- Session information and timestamps

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
kodegen_tools_introspection = "0.1"
```

## Usage

### Running the Server

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/kodegen-introspection
```

### As a Library

```rust
use kodegen_tools_introspection::{InspectUsageStatsTool, InspectToolCallsTool};

// Create tools
let usage_tool = InspectUsageStatsTool::new(usage_tracker);
let history_tool = InspectToolCallsTool::new();

// Register with MCP routers
let (tool_router, prompt_router) = register_tool(
    tool_router,
    prompt_router,
    usage_tool,
);
```

### Example Client

```bash
cargo run --example introspection_demo
```

The example demonstrates:
- Connecting to the local HTTP server
- Calling both introspection tools
- Logging requests/responses
- Graceful shutdown

## Building

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## Architecture

- **Library** (`src/lib.rs`): Exports tool implementations
- **Binary** (`src/main.rs`): HTTP server using `kodegen_server_http`
- **History**: Persisted to `~/.config/kodegen-mcp/tool-history.jsonl` (last 1000 calls)

## Requirements

- Rust nightly toolchain
- Components: rustfmt, clippy
- Targets: x86_64-apple-darwin, wasm32-unknown-unknown

## License

Licensed under either of:

- Apache License, Version 2.0
- MIT License

at your option.

## Links

- [Homepage](https://kodegen.ai)
- [Repository](https://github.com/cyrup-ai/kodegen-tools-introspection)
