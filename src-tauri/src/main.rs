// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// --- 数据结构 ---

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ParsedEvent {
    time: String,
    role: String,   // user, assistant, system (tool_output)
    content: String,
    event_type: String, // message, tool_result
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

// --- 核心逻辑函数 ---

/// 查找最近的 Session 文件
fn find_latest_session_file_internal() -> Option<PathBuf> {
    let home_dir = dirs::home_dir()?;
    let base_pattern = home_dir.join(".claude/projects/**/*.jsonl");
    let pattern_str = base_pattern.to_str()?;

    let mut latest_file: Option<PathBuf> = None;
    let mut latest_time = SystemTime::UNIX_EPOCH;

    if let Ok(paths) = glob::glob(pattern_str) {
        for entry in paths.filter_map(Result::ok) {
            if let Ok(metadata) = std::fs::metadata(&entry) {
                if let Ok(modified) = metadata.modified() {
                    if modified > latest_time {
                        latest_time = modified;
                        latest_file = Some(entry);
                    }
                }
            }
        }
    }
    latest_file
}

/// 解析 JSONL 文件 (复刻 Python 逻辑)
fn parse_jsonl_internal(file_path: &str) -> Result<Vec<ParsedEvent>, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut events = Vec::new();

    for line in reader.lines() {
        let line_content = line.map_err(|e| e.to_string())?;
        if line_content.trim().is_empty() { continue; }

        let json: Value = serde_json::from_str(&line_content).unwrap_or(Value::Null);
        if json.is_null() { continue; }

        let timestamp = json["timestamp"].as_str().unwrap_or("").to_string();

        // 1. 提取 Message (User / Assistant)
        if let Some(msg) = json.get("message") {
            let role = msg["role"].as_str().unwrap_or("unknown").to_string();
            let mut final_text = String::new();

            // 处理 content 是列表的情况 (新版格式)
            if let Some(content_arr) = msg["content"].as_array() {
                for part in content_arr {
                    let p_type = part["type"].as_str().unwrap_or("");
                    if p_type == "text" {
                        final_text.push_str(part["text"].as_str().unwrap_or(""));
                        final_text.push('\n');
                    } else if p_type == "tool_use" {
                        let name = part["name"].as_str().unwrap_or("unknown");
                        final_text.push_str(&format!("[操作] 调用工具: {}\n", name));
                        if let Some(input) = part.get("input") {
                            if let Some(path) = input.get("path").and_then(|v| v.as_str()) {
                                final_text.push_str(&format!("  - 路径: {}\n", path));
                            }
                        }
                    }
                }
            } 
            // 处理 content 是字符串的情况 (旧格式)
            else if let Some(content_str) = msg["content"].as_str() {
                final_text = content_str.to_string();
            }

            if !final_text.trim().is_empty() {
                events.push(ParsedEvent {
                    time: timestamp.clone(),
                    role,
                    content: final_text,
                    event_type: "message".to_string(),
                });
            }
        }

        // 2. 提取 ToolResult (上下文关键)
        if let Some(tool_res) = json.get("toolUseResult") {
            let mut res_str = String::new();

            if let Some(filenames) = tool_res.get("filenames").and_then(|v| v.as_array()) {
                // ls / grep 结果
                let preview: Vec<String> = filenames.iter()
                    .take(3)
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .collect();
                res_str = format!("[工具结果] 找到 {} 个文件: {:?}...", filenames.len(), preview);
            } else if let Some(file_info) = tool_res.get("file") {
                // read_file 结果
                let path = file_info["filePath"].as_str().unwrap_or("unknown");
                let content = file_info["content"].as_str().unwrap_or("");
                // 截断
                let snippet: String = content.chars().take(300).collect();
                res_str = format!("[工具结果] 已读取文件 {}。\n文件片段: {}...", path, snippet.replace('\n', " "));
            } else if let Some(result) = tool_res.get("result") {
                // 命令行执行结果
                let raw = result.as_str().unwrap_or("");
                let limit = if raw.to_lowercase().contains("error") { 800 } else { 300 };
                let snippet: String = raw.chars().take(limit).collect();
                res_str = format!("[工具结果] 输出: {}...", snippet);
            } else {
                // 其他情况
                res_str = format!("[工具结果] {}", tool_res.to_string().chars().take(200).collect::<String>());
            }

            if !res_str.is_empty() {
                events.push(ParsedEvent {
                    time: timestamp,
                    role: "system".to_string(),
                    content: res_str,
                    event_type: "tool_result".to_string(),
                });
            }
        }
    }

    // 按时间排序
    events.sort_by(|a, b| a.time.cmp(&b.time));
    Ok(events)
}

// --- Tauri Commands ---

#[tauri::command]
fn get_latest_session_path() -> Result<String, String> {
    find_latest_session_file_internal()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "未找到会话文件".to_string())
}

#[tauri::command]
fn parse_session_file(file_path: String) -> Result<Vec<ParsedEvent>, String> {
    parse_jsonl_internal(&file_path)
}

#[tauri::command]
async fn analyze_session(file_path: String, goal: String, api_key: String) -> Result<String, String> {
    // 1. 解析
    let events = parse_jsonl_internal(&file_path)?;
    
    if events.is_empty() {
        return Err("解析结果为空".to_string());
    }

    // 2. 格式化上下文 (取最后 20 条)
    let start_idx = if events.len() > 20 { events.len() - 20 } else { 0 };
    let recent_events = &events[start_idx..];
    
    let context_str = recent_events.iter().map(|e| {
        format!("[{}] {}:\n{}", e.time, e.role.to_uppercase(), e.content)
    }).collect::<Vec<String>>().join("\n--------------------\n");

    // 3. 调用 Gemini API
    let system_prompt = r#"
你是一个 Claude Code 结对编程助手。请分析下方的会话日志（包含用户指令、Claude 的操作、以及工具返回的文件内容/报错）。

任务：
1. **判断焦点 (Focus Check)**：Claude 是否陷入了死循环？是否在反复读取无关文件？是否无视了报错？
2. **生成提示词 (Prompt Generation)**：为用户写一段可以直接发送给 Claude 的**中文指令**。
   - 如果 Claude 走偏了：写一段严厉的纠正指令。
   - 如果 Claude 做得对：写一段推进下一步的指令，并引用刚才读取到的文件上下文（例如："基于刚才读取的 main.py..."）。
   
输出格式：
---
【状态】: [正常 / 迷失 / 报错循环]
【分析】: (简短分析当前情况)
【建议提示词】:
(你的 Prompt 内容)
---
"#;

    let full_text = format!("{}\n\n== 会话日志 ==\n{}\n\n== 用户当前目标 ==\n{}", system_prompt, context_str, goal);
    let url = format!("https://aiplatform.googleapis.com/v1/publishers/google/models/gemini-2.0-flash-lite:streamGenerateContent?key={}", api_key);

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{ "text": full_text }]
        }]
    });

    let resp = client.post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Network Error: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("API Error: {}", resp.status()));
    }

    let json_res: Value = resp.json().await.map_err(|e| format!("Json Parse Error: {}", e))?;
    
    // 解析 Gemini 响应 (兼容 streamGenerateContent 返回数组的情况)
    let mut final_text = String::new();
    if let Some(arr) = json_res.as_array() {
        for item in arr {
            if let Some(candidates) = item.get("candidates") {
                if let Some(content) = candidates[0].get("content") {
                    if let Some(parts) = content["parts"].as_array() {
                        if let Some(text) = parts[0]["text"].as_str() {
                            final_text.push_str(text);
                        }
                    }
                }
            }
        }
    } else {
        // 尝试解析单对象 (备用)
         if let Some(candidates) = json_res.get("candidates") {
             final_text = candidates[0]["content"]["parts"][0]["text"].as_str().unwrap_or("").to_string();
         }
    }

    Ok(final_text)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_latest_session_path,
            parse_session_file,
            analyze_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}