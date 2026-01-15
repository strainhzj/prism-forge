# 调试空会话内容问题

## 问题描述

用户报告:点击存在内容的会话文件时,前端显示"暂无内容"或"该会话文件为空或格式不正确"。

## 调试步骤

### 1. 重新编译并启动应用

```bash
# 停止所有正在运行的 dev 服务器
# 然后重新启动
npm run tauri dev
```

### 2. 查看后端调试日志

启动应用后,在终端中会看到类似以下的调试输出:

```
[DEBUG] 解析到 XX 个 JSONL 条目
[DEBUG] 转换后得到 XX 个消息对象 (view_level: Full)
[DEBUG] 过滤后得到 XX 个消息
```

**关键信息:**

1. **解析到的 JSONL 条目数** - 如果为 0,说明文件解析失败
2. **转换后的消息对象数** - 如果为 0,说明没有符合条件的消息类型
3. **过滤后的消息数** - 如果为 0,说明所有消息都被视图等级过滤掉了

### 3. 常见问题诊断

#### 问题 1: 解析到 0 个 JSONL 条目

**可能原因:**
- 会话文件格式不正确
- 文件路径错误
- 文件编码问题

**调试输出示例:**
```
[DEBUG] 解析到 0 个 JSONL 条目
```

**解决方案:**
1. 检查文件路径是否正确
2. 打开文件查看内容格式
3. 确认文件是 JSONL 格式(每行一个 JSON 对象)

#### 问题 2: 转换后得到 0 个消息对象

**可能原因:**
- JSONL 条目中没有 `type: "message"` 的记录
- `uuid` 字段缺失
- `role` 字段缺失

**调试输出示例:**
```
[DEBUG] 解析到 150 个 JSONL 条目
[DEBUG] 转换后得到 0 个消息对象 (view_level: Full)
```

**解决方案:**
查看会话文件的实际内容,检查:
- 是否有 `type: "message"` 的条目
- 是否有 `uuid` 和 `role` 字段

#### 问题 3: 过滤后得到 0 个消息

**可能原因:**
- 视图等级与消息类型不匹配
- 消息的 `msg_type` 字段值不符合预期

**调试输出示例:**
```
[DEBUG] 转换后得到 50 个消息对象 (view_level: AssistantOnly)
[DEBUG] 过滤后得到 0 个消息
[DEBUG] 原始消息示例:
  [0]: msg_type=user, uuid=12345678
  [1]: msg_type=user, uuid=87654321
  [2]: msg_type=user, uuid=abcdef12
```

**解决方案:**
从调试输出可以看到原始消息的 `msg_type` 值:
- 如果选择 `AssistantOnly` 视图,但所有消息都是 `user` 类型,就会得到空结果
- 尝试切换到 `Full` 或 `Conversation` 视图查看所有消息

### 4. 检查会话文件格式

可以在终端中手动查看会话文件内容:

```bash
# Windows PowerShell
Get-Content "C:\path\to\session.jsonl" -Head 5

# Windows CMD
powershell -Command "Get-Content 'C:\path\to\session.jsonl' -Head 5"
```

**预期的文件格式示例:**
```json
{"type":"message","uuid":"abc123","role":"user","timestamp":"2025-01-15T10:00:00Z","content":"你好"}
{"type":"message","uuid":"def456","role":"assistant","timestamp":"2025-01-15T10:00:05Z","content":"你好!有什么我可以帮助你的吗?"}
{"type":"message","uuid":"ghi789","role":"user","timestamp":"2025-01-15T10:00:10Z","content":"帮我写一段代码"}
```

### 5. 前端浏览器控制台

1. 打开应用
2. 按 `F12` 打开浏览器开发者工具
3. 切换到 `Console` 标签页
4. 点击会话文件
5. 查看是否有 JavaScript 错误

**常见的前端错误:**
- `TypeError: Cannot read property 'map' of undefined` - 数据格式不匹配
- `NetworkError` - Tauri 命令调用失败

### 6. 提供诊断信息

如果问题仍未解决,请提供以下信息:

1. **完整的后端调试日志** (包括所有 `[DEBUG]` 行)
2. **会话文件的前 5 行内容** (脱敏处理)
3. **当前选择的视图等级**
4. **浏览器控制台的错误信息** (如果有)

## 快速测试

创建一个简单的测试会话文件:

```json
{"type":"message","uuid":"test-user-1","role":"user","timestamp":"2025-01-15T10:00:00Z","content":"测试用户消息"}
{"type":"message","uuid":"test-assistant-1","role":"assistant","timestamp":"2025-01-15T10:00:01Z","content":"测试助手回复"}
```

保存为 `test-session.jsonl`,然后在应用中打开此文件,查看是否能正常显示。

## 代码变更说明

在 `src-tauri/src/commands.rs` 的 `cmd_get_messages_by_level` 函数中添加了调试日志:

```rust
#[cfg(debug_assertions)]
eprintln!("[DEBUG] 解析到 {} 个 JSONL 条目", entries.len());

#[cfg(debug_assertions)]
eprintln!("[DEBUG] 转换后得到 {} 个消息对象 (view_level: {:?})", messages.len(), view_level);

#[cfg(debug_assertions)]
{
    eprintln!("[DEBUG] 过滤后得到 {} 个消息", filtered_messages.len());
    if filtered_messages.is_empty() && !messages.is_empty() {
        eprintln!("[DEBUG] 原始消息示例:");
        for (i, msg) in messages.iter().take(3).enumerate() {
            eprintln!("  [{}]: msg_type={}, uuid={}", i, msg.msg_type, &msg.uuid[..8]);
        }
    }
}
```

这些日志仅在调试模式(`npm run tauri dev`)下输出,不会影响生产构建。
