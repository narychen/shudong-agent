use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;

use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use rig_core::client::{CompletionClient, ProviderClient};
use rig_core::completion::Prompt;
use rig_core::providers::{anthropic, ollama, openai};
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Runtime::new().expect("Failed to create Tokio runtime")
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    #[serde(rename = "OpenAI")]
    OpenAI,
    #[serde(rename = "Anthropic")]
    Anthropic,
    #[serde(rename = "Ollama")]
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub provider: ProviderType,
    pub api_key: String,
    pub model: String,
    pub system_prompt: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            provider: ProviderType::OpenAI,
            api_key: String::new(),
            model: "gpt-4".to_string(),
            system_prompt: "你是一个专业的编码助手。你可以使用以下工具来完成任务：\n- read_file(path): 读取文件内容\n- write_file(path, content): 写入文件\n- search_code(pattern): 搜索代码\n- execute_shell(command): 执行Shell命令\n\n当用户要求你做某事时，先思考需要使用哪些工具，然后调用工具完成任务。".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

struct AgentState {
    config: AgentConfig,
    is_initialized: bool,
    last_error: Option<String>,
    conversation_history: Vec<ConversationMessage>,
}

static AGENT_STATE: Lazy<Mutex<AgentState>> = Lazy::new(|| {
    Mutex::new(AgentState {
        config: AgentConfig::default(),
        is_initialized: false,
        last_error: None,
        conversation_history: Vec::new(),
    })
});

fn into_raw_string(value: String) -> *mut c_char {
    CString::new(value).unwrap().into_raw()
}

unsafe fn read_c_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    CStr::from_ptr(ptr).to_string_lossy().into_owned()
}

// 工具实现
fn read_file(path: &str) -> String {
    match std::fs::read_to_string(path) {
        Ok(content) => format!("成功读取文件 {}:\n{}", path, content),
        Err(e) => format!("读取文件 {} 失败: {}", path, e),
    }
}

fn write_file(path: &str, content: &str) -> String {
    match std::fs::write(path, content) {
        Ok(_) => format!("成功写入文件 {}", path),
        Err(e) => format!("写入文件 {} 失败: {}", path, e),
    }
}

fn search_code(pattern: &str) -> String {
    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => return format!("正则表达式错误: {}", e),
    };

    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if regex.is_match(line) {
                            results.push(format!(
                                "{}:{}: {}",
                                path.display(),
                                line_num + 1,
                                line
                            ));
                        }
                    }
                }
            }
        }
    }

    if results.is_empty() {
        "未找到匹配的代码".to_string()
    } else {
        format!("找到 {} 个匹配:\n{}", results.len(), results.join("\n"))
    }
}

fn execute_shell(command: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(&["/C", command])
            .output()
    } else {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            format!("命令执行完成\n输出:\n{}\n错误:\n{}", stdout, stderr)
        }
        Err(e) => format!("执行命令失败: {}", e),
    }
}

// 解析工具调用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub args: std::collections::HashMap<String, String>,
}

fn parse_tool_calls(response: &str) -> Vec<ToolCall> {
    let mut tools = Vec::new();

    // 简单的工具调用解析
    // 格式: <tool name="read_file" path="/path/to/file"/>
    let tool_regex = Regex::new(r#"<tool\s+name="(\w+)"([^>]*)/?>"#).unwrap();

    for cap in tool_regex.captures_iter(response) {
        let name = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let attrs_str = cap.get(2).map(|m| m.as_str()).unwrap_or("");

        let mut args = std::collections::HashMap::new();
        let attr_regex = Regex::new(r#"(\w+)="([^"]*)""#).unwrap();
        for attr_cap in attr_regex.captures_iter(attrs_str) {
            if let (Some(key), Some(val)) = (attr_cap.get(1), attr_cap.get(2)) {
                args.insert(key.as_str().to_string(), val.as_str().to_string());
            }
        }

        tools.push(ToolCall { name, args });
    }

    tools
}

fn execute_tool(tool: &ToolCall) -> String {
    match tool.name.as_str() {
        "read_file" => {
            let path = tool.args.get("path").map(|s| s.as_str()).unwrap_or("");
            read_file(path)
        }
        "write_file" => {
            let path = tool.args.get("path").map(|s| s.as_str()).unwrap_or("");
            let content = tool.args.get("content").map(|s| s.as_str()).unwrap_or("");
            write_file(path, content)
        }
        "search_code" => {
            let pattern = tool.args.get("pattern").map(|s| s.as_str()).unwrap_or("");
            search_code(pattern)
        }
        "execute_shell" => {
            let command = tool.args.get("command").map(|s| s.as_str()).unwrap_or("");
            execute_shell(command)
        }
        _ => format!("未知工具: {}", tool.name),
    }
}

fn process_task_sync(task: &str) -> Result<String> {
    let mut state = AGENT_STATE.lock();

    if !state.is_initialized {
        return Err(anyhow::anyhow!("Agent not initialized. Call shudong_init first."));
    }

    // 添加用户消息到对话历史
    state.conversation_history.push(ConversationMessage {
        role: "user".to_string(),
        content: task.to_string(),
    });

    let config = state.config.clone();
    let history = state.conversation_history.clone();
    drop(state);

    let task_str = task.to_string();

    RUNTIME.block_on(async move {
        // 构建对话消息
        let mut full_prompt = String::new();
        for msg in &history {
            full_prompt.push_str(&format!("{}: {}\n", msg.role, msg.content));
        }

        let response = match config.provider {
            ProviderType::OpenAI => {
                std::env::set_var("OPENAI_API_KEY", &config.api_key);
                let client = openai::Client::from_env()
                    .map_err(|e| anyhow::anyhow!("OpenAI client error: {}", e))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .build();
                agent
                    .prompt(&full_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("OpenAI prompt error: {}", e))?
            }
            ProviderType::Anthropic => {
                let client = anthropic::Client::new(&config.api_key)
                    .map_err(|e| anyhow::anyhow!("Anthropic client error: {}", e))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .build();
                agent
                    .prompt(&full_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Anthropic prompt error: {}", e))?
            }
            ProviderType::Ollama => {
                let client = ollama::Client::new(rig_core::client::Nothing)
                    .map_err(|e| anyhow::anyhow!("Ollama client error: {}", e))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .build();
                agent
                    .prompt(&full_prompt)
                    .await
                    .map_err(|e| anyhow::anyhow!("Ollama prompt error: {}", e))?
            }
        };

        // 解析和执行工具调用
        let tools = parse_tool_calls(&response);
        let mut final_response = response.clone();

        if !tools.is_empty() {
            let mut tool_results = String::new();
            for tool in tools {
                let result = execute_tool(&tool);
                tool_results.push_str(&format!("\n工具 {}: {}\n", tool.name, result));
            }

            // 如果有工具调用，再次调用LLM来处理工具结果
            let tool_response_prompt = format!(
                "用户的任务: {}\n\n我使用了以下工具:{}\n\n现在请基于工具结果总结你的回答。",
                task_str, tool_results
            );

            final_response = match config.provider {
                ProviderType::OpenAI => {
                    std::env::set_var("OPENAI_API_KEY", &config.api_key);
                    let client = openai::Client::from_env()?;
                    let agent = client
                        .agent(&config.model)
                        .preamble(&config.system_prompt)
                        .build();
                    agent.prompt(&tool_response_prompt).await?
                }
                ProviderType::Anthropic => {
                    let client = anthropic::Client::new(&config.api_key)?;
                    let agent = client
                        .agent(&config.model)
                        .preamble(&config.system_prompt)
                        .build();
                    agent.prompt(&tool_response_prompt).await?
                }
                ProviderType::Ollama => {
                    let client = ollama::Client::new(rig_core::client::Nothing)?;
                    let agent = client
                        .agent(&config.model)
                        .preamble(&config.system_prompt)
                        .build();
                    agent.prompt(&tool_response_prompt).await?
                }
            };
        }

        // 保存助手回复到对话历史
        let mut state = AGENT_STATE.lock();
        state.conversation_history.push(ConversationMessage {
            role: "assistant".to_string(),
            content: final_response.clone(),
        });

        Ok(final_response)
    })
}

#[no_mangle]
pub extern "C" fn shudong_init(config_json: *const c_char) -> *mut c_char {
    let config_str = unsafe { read_c_string(config_json) };

    let config: AgentConfig = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            let mut state = AGENT_STATE.lock();
            state.last_error = Some(format!("Invalid config: {}", e));
            return into_raw_string(
                serde_json::json!({"success": false, "error": e.to_string()}).to_string(),
            );
        }
    };

    let mut state = AGENT_STATE.lock();
    state.config = config;
    state.is_initialized = true;
    state.last_error = None;
    state.conversation_history.clear();

    into_raw_string(serde_json::json!({"success": true}).to_string())
}

#[no_mangle]
pub extern "C" fn shudong_process_task(task: *const c_char) -> *mut c_char {
    let task = unsafe { read_c_string(task) };

    match process_task_sync(&task) {
        Ok(response) => into_raw_string(
            serde_json::json!({"success": true, "response": response}).to_string(),
        ),
        Err(e) => {
            let mut state = AGENT_STATE.lock();
            state.last_error = Some(e.to_string());
            into_raw_string(
                serde_json::json!({"success": false, "error": e.to_string()}).to_string(),
            )
        }
    }
}

#[no_mangle]
pub extern "C" fn shudong_get_status() -> *mut c_char {
    let state = AGENT_STATE.lock();
    let status = if state.is_initialized {
        serde_json::json!({
            "initialized": true,
            "provider": format!("{:?}", state.config.provider),
            "model": state.config.model,
            "history_length": state.conversation_history.len()
        })
    } else {
        serde_json::json!({"initialized": false})
    };
    into_raw_string(status.to_string())
}

#[no_mangle]
pub extern "C" fn shudong_get_last_error() -> *mut c_char {
    let state = AGENT_STATE.lock();
    let error = state.last_error.clone().unwrap_or_default();
    into_raw_string(error)
}

#[no_mangle]
pub extern "C" fn shudong_clear_history() -> *mut c_char {
    let mut state = AGENT_STATE.lock();
    state.conversation_history.clear();
    into_raw_string(serde_json::json!({"success": true}).to_string())
}

#[no_mangle]
pub extern "C" fn shudong_string_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_roundtrip() {
        let s = "Hello, world!";
        let ptr = into_raw_string(s.to_string());
        let result = unsafe { read_c_string(ptr) };
        unsafe { shudong_string_free(ptr) };
        assert_eq!(s, result);
    }

    #[test]
    fn test_tool_parsing() {
        let response = r#"我来为你读取这个文件。<tool name="read_file" path="/tmp/test.txt"/>"#;
        let tools = parse_tool_calls(response);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "read_file");
        assert_eq!(tools[0].args.get("path"), Some(&"/tmp/test.txt".to_string()));
    }

    #[test]
    fn test_json_serialization() {
        let json = serde_json::json!({"success": true, "response": "test"});
        let s = json.to_string();
        assert!(s.contains("success"));
        assert!(s.contains("response"));
    }
}
