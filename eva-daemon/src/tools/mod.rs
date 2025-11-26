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
    r#"You have access to the following tools:

1. web_search(query: str) -> Search the internet using DuckDuckGo
   Example: {"name": "web_search", "arguments": {"query": "latest Rust news"}}

2. run_command(command: str) -> Execute a shell command on macOS
   Example: {"name": "run_command", "arguments": {"command": "ls -la ~/Documents"}}

3. run_applescript(script: str) -> Execute AppleScript for system automation
   Example: {"name": "run_applescript", "arguments": {"script": "tell application \"Music\" to play"}}

4. read_file(path: str) -> Read contents of a file
   Example: {"name": "read_file", "arguments": {"path": "/Users/aarya/file.txt"}}

5. write_file(path: str, content: str) -> Write content to a file
   Example: {"name": "write_file", "arguments": {"path": "/tmp/test.txt", "content": "Hello"}}

6. list_directory(path: str) -> List files in a directory
   Example: {"name": "list_directory", "arguments": {"path": "/Users/aarya"}}

To use a tool, respond with a JSON object in this format:
{"tool": {"name": "tool_name", "arguments": {...}}}

After using a tool, you'll receive the result and can provide a natural response to the user."#
}
