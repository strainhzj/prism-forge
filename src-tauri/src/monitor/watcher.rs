//! 文件监控器
//!
//! 使用 notify crate 监控 Claude 会话文件变更，并通过 Tauri Events 推送到前端

use anyhow::{Result, Context};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use serde::{Deserialize, Serialize};

/// 监控事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchEvent {
    /// 事件类型：created, modified, deleted
    pub kind: String,
    /// 文件路径
    pub path: String,
    /// 是否为 JSONL 文件
    pub is_jsonl: bool,
    /// 事件时间戳
    pub timestamp: String,
}

/// 会话文件监控器
///
/// 监控 ~/.claude/projects/ 目录下的文件变更
pub struct SessionWatcher {
    /// 监控目录
    watch_path: PathBuf,
    /// 事件发送通道
    tx: Sender<WatchEvent>,
    /// Tauri App Handle（用于发送事件到前端）
    #[allow(dead_code)]
    app_handle: AppHandle,
}

impl SessionWatcher {
    /// 创建新的监控器
    ///
    /// # 参数
    /// - `watch_path`: 要监控的目录路径
    /// - `app_handle`: Tauri App Handle
    ///
    /// # 返回
    /// 返回监控器实例或错误
    pub fn new(watch_path: PathBuf, app_handle: AppHandle) -> Result<Self> {
        let (tx, _rx) = mpsc::channel();

        Ok(Self {
            watch_path,
            tx,
            app_handle,
        })
    }

    /// 启动监控器
    ///
    /// 开始监控文件变更，并推送事件到前端
    ///
    /// # 返回
    /// 返回线程句柄或错误
    pub fn start(self) -> Result<thread::JoinHandle<()>> {
        // 创建事件接收通道
        let (event_tx, event_rx): (Sender<notify::Event>, Receiver<notify::Event>) = mpsc::channel();

        // 创建 notify watcher
        let mut watcher: RecommendedWatcher = Watcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = event_tx.send(event);
                }
            },
            notify::Config::default(),
        ).context("创建文件监控器失败")?;

        // 监控目标目录
        watcher.watch(&self.watch_path, RecursiveMode::Recursive)
            .context("设置监控目录失败")?;

        // 启动处理线程
        let handle = thread::spawn(move || {
            eprintln!("文件监控器已启动，监控目录: {:?}", self.watch_path);
            self.run_event_loop(event_rx);
        });

        Ok(handle)
    }

    /// 运行事件处理循环
    ///
    /// 处理文件变更事件，去重、防抖后推送到前端
    fn run_event_loop(self, event_rx: Receiver<notify::Event>) {
        let mut debounce_buffer: Vec<WatchEvent> = Vec::new();
        let mut last_event_time = std::time::Instant::now();
        let debounce_duration = Duration::from_secs(2);

        loop {
            match event_rx.recv_timeout(Duration::from_millis(500)) {
                Ok(event) => {
                    // 过滤非 JSONL 文件
                    if let Some(path) = event.paths.first() {
                        let is_jsonl = path.extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "jsonl")
                            .unwrap_or(false);

                        if !is_jsonl {
                            continue;
                        }

                        // 转换为 WatchEvent
                        let kind = match event.kind {
                            EventKind::Create(_) => "created",
                            EventKind::Modify(_) => "modified",
                            EventKind::Remove(_) => "deleted",
                            _ => "other",
                        }.to_string();

                        let watch_event = WatchEvent {
                            kind,
                            path: path.to_string_lossy().to_string(),
                            is_jsonl,
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        };

                        debounce_buffer.push(watch_event);
                        last_event_time = std::time::Instant::now();
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // 超时：检查是否超过防抖时间
                    if !debounce_buffer.is_empty()
                        && last_event_time.elapsed() >= debounce_duration {
                        // 防抖时间已过，批量处理事件
                        self.flush_events(&mut debounce_buffer);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("文件监控器通道已断开");
                    break;
                }
            }
        }
    }

    /// 批量处理事件
    ///
    /// 去重后推送到前端
    fn flush_events(&self, events: &mut Vec<WatchEvent>) {
        if events.is_empty() {
            return;
        }

        eprintln!("文件监控器: 处理 {} 个事件", events.len());

        // 去重：同一个文件的多个事件只保留最新的
        let mut unique_events: std::collections::HashMap<String, WatchEvent> = std::collections::HashMap::new();
        for event in events.drain(..) {
            unique_events.insert(event.path.clone(), event);
        }

        // 推送到前端
        for event in unique_events.values() {
            if let Err(e) = self.app_handle.emit("sessions-changed", &event) {
                eprintln!("推送事件到前端失败: {}", e);
            }
        }
    }
}

/// 获取 Claude 项目目录
///
/// 返回 ~/.claude/projects/ 路径
pub fn get_claude_projects_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("无法获取用户目录"))?;

    Ok(home_dir.join(".claude").join("projects"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_claude_projects_dir() {
        let dir = get_claude_projects_dir().unwrap();
        assert!(dir.ends_with(".claude/projects"));
    }
}
