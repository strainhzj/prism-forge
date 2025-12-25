import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

// 类型定义
interface ParsedEvent {
  time: string;
  role: string;
  content: string;
  event_type: string;
}

function App() {
  // 状态管理
  const [filePath, setFilePath] = useState("");
  const [apiKey, setApiKey] = useState(localStorage.getItem("gemini_key") || "");
  const [goal, setGoal] = useState("");
  
  const [parsedEvents, setParsedEvents] = useState<ParsedEvent[]>([]);
  const [analysisResult, setAnalysisResult] = useState("");
  const [loading, setLoading] = useState(false);
  const [analyzing, setAnalyzing] = useState(false);

  // 初始化：自动查找最近的文件
  useEffect(() => {
    autoDetectFile();
  }, []);

  // 保存 Key 到本地
  useEffect(() => {
    localStorage.setItem("gemini_key", apiKey);
  }, [apiKey]);

  const autoDetectFile = async () => {
    try {
      const path = await invoke<string>("get_latest_session_path");
      setFilePath(path);
      // 自动加载解析结果进行预览
      loadParsedEvents(path);
    } catch (e) {
      console.error(e);
      setFilePath(""); 
      // alert("未找到会话文件");
    }
  };

  const loadParsedEvents = async (path: string) => {
    if (!path) return;
    try {
      const events = await invoke<ParsedEvent[]>("parse_session_file", { filePath: path });
      setParsedEvents(events);
    } catch (e) {
      console.error("解析失败", e);
    }
  };

  const handleAnalyze = async () => {
    if (!filePath || !apiKey || !goal) {
      alert("请填写完整信息 (文件路径、API Key、目标)");
      return;
    }

    setAnalyzing(true);
    setAnalysisResult("");
    
    try {
      // 1. 先刷新一下解析列表，确保是最新的
      await loadParsedEvents(filePath);
      
      // 2. 调用 AI 分析
      const result = await invoke<string>("analyze_session", {
        filePath,
        goal,
        apiKey
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
      <h1>Claude Session Monitor</h1>

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
          <label>Gemini API Key:</label>
          <input 
            type="password"
            value={apiKey} 
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="AIzaSy..."
          />
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
            {parsedEvents.length === 0 && <p className="placeholder">No events parsed.</p>}
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