use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
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
    OpenAI,
    Anthropic,
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
            system_prompt: "你是一个专业的编码助手，擅长分析代码、提供优化建议、执行开发任务。".to_string(),
        }
    }
}

struct AgentState {
    config: AgentConfig,
    is_initialized: bool,
    last_error: Option<String>,
}

static AGENT_STATE: Lazy<Mutex<AgentState>> = Lazy::new(|| {
    Mutex::new(AgentState {
        config: AgentConfig::default(),
        is_initialized: false,
        last_error: None,
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

fn process_task_sync(task: &str) -> Result<String> {
    let state = AGENT_STATE.lock();
    
    if !state.is_initialized {
        return Err(anyhow::anyhow!("Agent not initialized. Call shudong_init first."));
    }
    
    let config = state.config.clone();
    drop(state);
    
    let task = task.to_string();
    
    RUNTIME.block_on(async move {
        match config.provider {
            ProviderType::OpenAI => {
                std::env::set_var("OPENAI_API_KEY", &config.api_key);
                let client = openai::Client::from_env()
                    .map_err(|e| anyhow::anyhow!("OpenAI client error: {}", e))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .build();
                agent.prompt(&task).await
                    .map_err(|e| anyhow::anyhow!("OpenAI prompt error: {}", e))
            }
            ProviderType::Anthropic => {
                let client = anthropic::Client::new(&config.api_key)
                    .map_err(|e| anyhow::anyhow!("Anthropic client error: {}", e))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .build();
                agent.prompt(&task).await
                    .map_err(|e| anyhow::anyhow!("Anthropic prompt error: {}", e))
            }
            ProviderType::Ollama => {
                let client = ollama::Client::new(rig_core::client::Nothing)
                    .map_err(|e| anyhow::anyhow!("Ollama client error: {}", e))?;
                let agent = client
                    .agent(&config.model)
                    .preamble(&config.system_prompt)
                    .build();
                agent.prompt(&task).await
                    .map_err(|e| anyhow::anyhow!("Ollama prompt error: {}", e))
            }
        }
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
            return into_raw_string(format!("{{\"success\": false, \"error\": \"{}\"}}", e));
        }
    };
    
    let mut state = AGENT_STATE.lock();
    state.config = config;
    state.is_initialized = true;
    state.last_error = None;
    
    into_raw_string("{\"success\": true}".to_string())
}

#[no_mangle]
pub extern "C" fn shudong_process_task(task: *const c_char) -> *mut c_char {
    let task = unsafe { read_c_string(task) };
    
    match process_task_sync(&task) {
        Ok(response) => into_raw_string(format!("{{\"success\": true, \"response\": \"{}\"}}", 
            response.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"))),
        Err(e) => {
            let mut state = AGENT_STATE.lock();
            state.last_error = Some(e.to_string());
            into_raw_string(format!("{{\"success\": false, \"error\": \"{}\"}}", e))
        }
    }
}

#[no_mangle]
pub extern "C" fn shudong_get_status() -> *mut c_char {
    let state = AGENT_STATE.lock();
    let status = if state.is_initialized {
        format!("{{\"initialized\": true, \"provider\": \"{:?}\", \"model\": \"{}\"}}", 
            state.config.provider, state.config.model)
    } else {
        "{\"initialized\": false}".to_string()
    };
    into_raw_string(status)
}

#[no_mangle]
pub extern "C" fn shudong_get_last_error() -> *mut c_char {
    let state = AGENT_STATE.lock();
    let error = state.last_error.clone().unwrap_or_default();
    into_raw_string(error)
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
    fn test_init_invalid_json() {
        let ptr = into_raw_string("invalid json".to_string());
        let result_ptr = unsafe { shudong_init(ptr) };
        let result = unsafe { read_c_string(result_ptr) };
        assert!(result.contains("success"));
        unsafe { shudong_string_free(ptr) };
        unsafe { shudong_string_free(result_ptr) };
    }
}
