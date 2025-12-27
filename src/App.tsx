import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import "./App.css";

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[App] ${action}`, ...args);
  }
}

// 类型定义
interface ParsedEvent {
  time: string;
  role: string;
  content: string;
  event_type: string;
}

function App() {
  const navigate = useNavigate();

  // 状态管理
  const [filePath, setFilePath] = useState("");
  const [goal, setGoal] = useState("");

  const [parsedEvents, setParsedEvents] = useState<ParsedEvent[]>([]);
  const [analysisResult, setAnalysisResult] = useState("");
  const [analyzing, setAnalyzing] = useState(false);
  const [parseError, setParseError] = useState("");  // 解析错误信息

  // F6 快捷键导航到设置页面
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'F6') {
      e.preventDefault();
      debugLog('keydown', 'F6 pressed, navigating to settings');
      navigate('/settings');
    }
  }, [navigate]);

  // 注册全局快捷键
  useEffect(() => {
    debugLog('useEffect', 'registering F6 shortcut');
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      debugLog('useEffect', 'unregistering F6 shortcut');
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);

  // 初始化：自动查找最近的文件
  useEffect(() => {
    autoDetectFile();
  }, []);

  const autoDetectFile = async () => {
    try {
      const path = await invoke<string>("get_latest_session_path");
      setFilePath(path);
      setParseError("");  // 清空之前的错误
      // 自动加载解析结果进行预览
      loadParsedEvents(path);
    } catch (e) {
      const errorMsg = `自动检测文件失败: ${e}`;
      console.error(errorMsg);
      setParseError(errorMsg);
      setFilePath("");
    }
  };

  const loadParsedEvents = async (path: string) => {
    if (!path) return;
    try {
      setParseError("");  // 清空之前的错误
      const events = await invoke<ParsedEvent[]>("parse_session_file", { filePath: path });
      setParsedEvents(events);
      debugLog("解析成功", `获取到 ${events.length} 个事件`);
    } catch (e) {
      const errorMsg = `解析会话文件失败: ${e}`;
      console.error(errorMsg);
      setParseError(errorMsg);
      setParsedEvents([]);  // 清空之前的结果
    }
  };

  const handleAnalyze = async () => {
    if (!filePath || !goal) {
      alert("请填写完整信息 (文件路径、目标)");
      return;
    }

    setAnalyzing(true);
    setAnalysisResult("");

    try {
      // 1. 先刷新一下解析列表，确保是最新的
      await loadParsedEvents(filePath);

      // 2. 调用新的 optimize_prompt 命令（使用 LLMClientManager 中配置的 API）
      const result = await invoke<string>("optimize_prompt", {
        sessionFile: filePath,
        goal
      });
      setAnalysisResult(result);
    } catch (e) {
      setAnalysisResult(`Error: ${e}`);
    } finally {
      setAnalyzing(false);
    }
  };

  return (
    <div className="container">
      <div className="header-row">
        <h1>Claude Session Monitor</h1>
        <button
          className="settings-btn"
          onClick={() => navigate('/settings')}
          title="配置 LLM API 提供商"
        >
          ⚙️ 设置 (F6)
        </button>
      </div>

      <div className="config-section">
        <div className="input-group">
          <label>Session File:</label>
          <div className="row">
            <input
              value={filePath}
              onChange={(e) => setFilePath(e.target.value)}
              placeholder="Path to .jsonl file"
            />
            <button onClick={autoDetectFile}>Auto Detect</button>
          </div>
        </div>

        <div className="input-group">
          <label>Next Goal:</label>
          <textarea
            value={goal}
            onChange={(e) => setGoal(e.target.value)}
            placeholder="e.g. Fix the null pointer exception in user service"
            rows={2}
          />
        </div>

        <div className="info-note">
          <small>
            ℹ️ API 配置已移至设置页 (F6)。请确保已配置活跃的 LLM 提供商。
          </small>
        </div>

        <button
          className="primary-btn"
          onClick={handleAnalyze}
          disabled={analyzing}
        >
          {analyzing ? "Analyzing..." : "Analyze & Generate Prompt"}
        </button>
      </div>

      <div className="main-content">
        {/* 左侧：AI 建议结果 */}
        <div className="result-panel">
          <h2>AI Analysis Result</h2>
          <div className="result-box">
            {analysisResult ? (
              <pre>{analysisResult}</pre>
            ) : (
              <p className="placeholder">Result will appear here...</p>
            )}
          </div>
        </div>

        {/* 右侧：日志解析预览 (Debug 模式) */}
        <div className="debug-panel">
          <div className="debug-header">
            <h2>Session Log ({parsedEvents.length})</h2>
            <button onClick={() => loadParsedEvents(filePath)} className="sm-btn">Refresh</button>
          </div>
          <div className="events-list">
            {parseError && <p className="error-message">{parseError}</p>}
            {parsedEvents.length === 0 && !parseError && <p className="placeholder">No events parsed.</p>}
            {parsedEvents.slice().reverse().map((ev, i) => (
              <div key={i} className={`event-card ${ev.event_type}`}>
                <div className="event-meta">
                  <span className="role">{ev.role.toUpperCase()}</span>
                  <span className="time">{ev.time.split("T")[1]?.substring(0,8)}</span>
                </div>
                <div className="event-content">
                  {ev.content.length > 200 
                    ? ev.content.substring(0, 200) + "..." 
                    : ev.content}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;