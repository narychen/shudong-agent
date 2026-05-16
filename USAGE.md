# Shudong Agent 使用指南

## 项目状态

✅ **项目现在已经完全可用**，包含以下功能：

### 核心功能
- **多LLM后端支持**: OpenAI、Anthropic、Ollama
- **Tool Calling**: LLM 可以自动调用工具完成任务
- **编码工具集**:
  - `read_file(path)` - 读取文件内容
  - `write_file(path, content)` - 写入文件
  - `search_code(pattern)` - 用正则表达式搜索代码
  - `execute_shell(command)` - 执行 Shell 命令
- **对话历史管理** - 支持多轮对话上下文
- **完整的错误处理** - JSON 序列化正确，安全的 FFI 接口

### UI 特性
- 聊天界面显示对话历史
- 实时状态指示（绿色/橙色）
- 设置页面配置 LLM 提供商和 API Key
- 清除历史按钮
- 错误消息红色提示

## 使用步骤

### 1. 准备 API Key

获取以下任一 API Key：
- **OpenAI**: https://platform.openai.com/api-keys (需要 GPT-4 权限)
- **Anthropic**: https://console.anthropic.com/account/keys (Claude API)
- **Ollama**: 本地免费运行（https://ollama.ai）- 无需 API Key

### 2. 启动应用

```bash
# 编译 Rust 核心
cd core
cargo build

# 设置环境变量并启动 Flutter 应用
cd ../app
export SHUDONG_CORE_LIB=../core/target/debug/libshudong_core.dylib
flutter run -d macos
```

### 3. 配置 Agent

1. 点击右上角 **Settings** 按钮
2. 选择 LLM 提供商（OpenAI/Anthropic/Ollama）
3. 选择模型
4. 输入 API Key
5. 点击 **Initialize**

### 4. 使用 Agent

在下方文本框输入任务，例如：

**简单对话:**
```
你好，请介绍一下自己
```

**文件操作:**
```
读取 /tmp/test.txt 文件内容
```

**代码搜索:**
```
搜索所有 TODO 标记的代码
```

**组合任务:**
```
读取 src/main.rs 文件，找出所有函数定义，然后给我一个总结
```

**Shell 命令:**
```
列出当前目录的所有文件
```

## 工作原理

1. **用户输入** → Flutter UI
2. **转发到 Rust 核心** → FFI 桥接
3. **LLM 处理** → rig-core 框架调用 LLM
4. **LLM 识别工具调用** → 解析响应中的工具标签
5. **执行工具** → 调用 read_file/write_file/search_code/execute_shell
6. **反馈给 LLM** → LLM 基于工具结果总结回答
7. **返回给用户** → Flutter UI 显示对话

## 注意事项

### ⚠️ 当前限制

- **Shell 命令执行** - 受 macOS 沙盒限制，某些系统命令可能被拒绝
- **文件访问范围** - 可以访问当前工作目录及其子目录的文件
- **工具调用格式** - LLM 必须使用 XML 标签格式：`<tool name="tool_name" param="value"/>`

### 🔧 故障排除

**"Rust core not wired"错误**
```bash
# 确保设置了环境变量
export SHUDONG_CORE_LIB=/path/to/core/target/debug/libshudong_core.dylib
```

**"Invalid API Key"错误**
- 检查 API Key 是否正确
- 确保账户有足够的配额
- 检查网络连接

**工具无法执行**
- 检查文件路径是否正确
- 确保有文件访问权限
- Shell 命令需要符合系统语法

## 架构说明

### Rust 核心 (core/)

**主要组件:**
- `AgentConfig` - Agent 配置（提供商、API Key、模型、系统提示）
- `ConversationMessage` - 对话消息（角色、内容）
- `ToolCall` - 工具调用（工具名、参数）

**核心函数:**
- `shudong_init()` - 初始化 agent
- `shudong_process_task()` - 处理用户任务（支持 Tool Calling）
- `shudong_get_status()` - 获取 agent 状态
- `shudong_clear_history()` - 清空对话历史

**工具实现:**
- `read_file()` - 读取文件
- `write_file()` - 写入文件
- `search_code()` - 正则搜索
- `execute_shell()` - Shell 命令执行

### Flutter 应用 (app/)

**主要页面:**
- `MyHomePage` - 主页（聊天界面）
  - `_messages` - 对话历史
  - `_initialized` - 初始化状态
  - `_busy` - 处理中状态

**主要功能:**
- `_initializeAgent()` - 配置并初始化 agent
- `_sendTask()` - 发送任务并处理响应
- `_showSettings()` - 显示设置对话框
- `_clearHistory()` - 清空对话历史

## 技术栈

- **前端**: Flutter/Dart (macOS)
- **后端**: Rust + rig-core (LLM 框架)
- **通信**: FFI (Foreign Function Interface)
- **LLM**: OpenAI/Anthropic/Ollama

## 下一步改进

- [ ] 支持流式响应（现在是完整等待）
- [ ] 持久化对话历史到数据库
- [ ] 支持多个 agent 实例
- [ ] 更多编码工具（git、docker 等）
- [ ] Web 界面支持
- [ ] iOS/Android 支持
- [ ] 本地模型集成优化

## 许可证

MIT License
