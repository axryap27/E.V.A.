pub mod web_search;
pub mod system_control;
pub mod file_ops;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool call request from LLM
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

/// Tool execution result
#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
}

/// Execute a tool call and return the result
pub async fn execute_tool(tool_call: &ToolCall) -> Result<ToolResult> {
    match tool_call.name.as_str() {
        "web_search" => web_search::execute(&tool_call.arguments).await,
        "run_command" => system_control::execute_command(&tool_call.arguments).await,
        "run_applescript" => system_control::execute_applescript(&tool_call.arguments).await,
        "read_file" => file_ops::read_file(&tool_call.arguments).await,
        "write_file" => file_ops::write_file(&tool_call.arguments).await,
        "list_directory" => file_ops::list_directory(&tool_call.arguments).await,
        _ => Ok(ToolResult {
            success: false,
            output: format!("Unknown tool: {}", tool_call.name),
        }),
    }
}

/// Available tools definition for LLM system prompt
pub fn get_tools_definition() -> &'static str {
    r#"IMPORTANT: You are a conversational assistant. Respond naturally to users. ONLY use tools when absolutely necessary for tasks you cannot answer directly.

DO NOT use tools for:
- Greetings, small talk, or general conversation
- Questions you can answer from your knowledge
- Simple requests that don't require external data or system actions

ONLY use tools when the user explicitly asks you to:
- Search the web for current information
- Control the computer or run commands
- Read, write, or manage files

Available tools:

1. web_search(query: str) -> Search the internet for current information
   Use when: User asks to search, find current news, or needs real-time data
   Example: {"tool": {"name": "web_search", "arguments": {"query": "latest Rust news"}}}

2. run_command(command: str) -> Execute a shell command on macOS
   Use when: User asks to run a command, check system info, or perform terminal actions
   Example: {"tool": {"name": "run_command", "arguments": {"command": "ls -la ~/Documents"}}}

3. run_applescript(script: str) -> Control macOS apps via AppleScript
   Use when: User asks to control apps like Music, Finder, or system functions
   Example: {"tool": {"name": "run_applescript", "arguments": {"script": "tell application \"Music\" to play"}}}

4. read_file(path: str) -> Read contents of a file
   Use when: User asks to read or show file contents
   Example: {"tool": {"name": "read_file", "arguments": {"path": "/Users/aarya/file.txt"}}}

5. write_file(path: str, content: str) -> Write content to a file
   Use when: User asks to create or save a file
   Example: {"tool": {"name": "write_file", "arguments": {"path": "/tmp/test.txt", "content": "Hello"}}}

6. list_directory(path: str) -> List files in a directory
   Use when: User asks to see what's in a folder
   Example: {"tool": {"name": "list_directory", "arguments": {"path": "/Users/aarya"}}}

To use a tool, output ONLY the JSON (nothing else):
{"tool": {"name": "tool_name", "arguments": {...}}}

For normal conversation, just respond naturally without any JSON."#
}
