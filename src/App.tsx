import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { ChevronLeft, ChevronRight, Settings, RefreshCw, FolderOpen } from "lucide-react";
import { ThemeToggle } from "./components/ThemeToggle";
import { cn } from "@/lib/utils";

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

// 自动刷新间隔（毫秒）
const AUTO_REFRESH_INTERVAL = 3000;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[App] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

interface ParsedEvent {
  time: string;
  role: string;
  content: string;
  event_type: string;
}

interface TimelineLog {
  id: string;
  timestamp: string;
  type: 'user' | 'assistant' | 'system';
  content: string;
}

// ==================== 主组件 ====================

function App() {
  const navigate = useNavigate();

  // 侧边栏折叠状态
  const [leftCollapsed, setLeftCollapsed] = useState(false);
  const [rightCollapsed, setRightCollapsed] = useState(false);

  // 原有状态
  const [filePath, setFilePath] = useState("");
  const [goal, setGoal] = useState("");
  const [parsedEvents, setParsedEvents] = useState<ParsedEvent[]>([]);
  const [analysisResult, setAnalysisResult] = useState("");
  const [analyzing, setAnalyzing] = useState(false);
  const [parseError, setParseError] = useState("");
  const [autoRefresh, setAutoRefresh] = useState(false);

  // 使用 ref 存储 loadParsedEvents 的引用
  const loadParsedEventsRef = useRef<(path: string) => Promise<void>>();

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

  // 自动刷新定时器
  useEffect(() => {
    if (autoRefresh && filePath) {
      debugLog('auto-refresh', '启动自动刷新，间隔:', AUTO_REFRESH_INTERVAL);
      const intervalId = setInterval(() => {
        if (filePath && loadParsedEventsRef.current) {
          debugLog('auto-refresh', '自动刷新中...');
          loadParsedEventsRef.current(filePath);
        }
      }, AUTO_REFRESH_INTERVAL);

      return () => {
        debugLog('auto-refresh', '清除自动刷新定时器');
        clearInterval(intervalId);
      };
    }
  }, [autoRefresh, filePath]);

  const autoDetectFile = async () => {
    try {
      const path = await invoke<string>("get_latest_session_path");
      setFilePath(path);
      setParseError("");
      loadParsedEvents(path);
    } catch (e) {
      const errorMsg = `自动检测文件失败: ${e}`;
      console.error(errorMsg);
      setParseError(errorMsg);
      setFilePath("");
    }
  };

  const loadParsedEvents = useCallback(async (path: string) => {
    if (!path) return;
    try {
      setParseError("");
      const events = await invoke<ParsedEvent[]>("parse_session_file", { filePath: path });
      setParsedEvents(events);
      debugLog("解析成功", `获取到 ${events.length} 个事件`);
    } catch (e) {
      const errorMsg = `解析会话文件失败: ${e}`;
      console.error(errorMsg);
      setParseError(errorMsg);
      setParsedEvents([]);
    }
  }, []);

  // 更新 ref 引用
  useEffect(() => {
    loadParsedEventsRef.current = loadParsedEvents;
  }, [loadParsedEvents]);

  const handleAnalyze = async () => {
    if (!filePath || !goal) {
      alert("请填写完整信息 (文件路径、目标)");
      return;
    }

    setAnalyzing(true);
    setAnalysisResult("");

    try {
      await loadParsedEvents(filePath);
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

  const toggleAutoRefresh = () => {
    setAutoRefresh(prev => !prev);
  };

  // 转换为时间线日志格式
  const timelineLogs: TimelineLog[] = parsedEvents.slice().reverse().map((ev, i) => ({
    id: `log-${i}`,
    timestamp: ev.time,
    type: ev.role.toLowerCase() === 'user' ? 'user' : 'assistant',
    content: ev.content.length > 150 ? ev.content.substring(0, 150) + "..." : ev.content
  }));

  return (
    <div className="flex h-screen bg-[#121212]" style={{ fontFamily: 'sans-serif' }}>
      {/* ==================== 左侧栏：项目目录 ==================== */}
      {!leftCollapsed && (
        <aside className="w-[240px] border-r shrink-0 flex flex-col" style={{ backgroundColor: '#1E1E1E', borderColor: '#333' }}>
          {/* 头部 */}
          <div className="flex items-center justify-between px-4 py-3 border-b" style={{ borderColor: '#333' }}>
            <h2 className="text-sm font-semibold" style={{ color: '#FF6B6B' }}>📁 项目</h2>
            <button
              onClick={() => setLeftCollapsed(true)}
              className="p-1 rounded hover:bg-[#333] transition-colors"
              title="折叠侧边栏"
            >
              <ChevronLeft className="h-4 w-4 text-gray-400" />
            </button>
          </div>

          {/* 项目列表 - 简化版 */}
          <div className="flex-1 overflow-y-auto p-3 space-y-2">
            {/* 当前跟踪项目 */}
            <div className="p-3 rounded-lg border" style={{ backgroundColor: 'rgba(255, 107, 107, 0.1)', borderColor: 'rgba(255, 107, 107, 0.3)' }}>
              <div className="flex items-center gap-2 mb-2">
                <FolderOpen className="h-4 w-4" style={{ color: '#FF6B6B' }} />
                <span className="text-sm font-medium" style={{ color: '#FF6B6B' }}>当前会话</span>
              </div>
              <p className="text-xs text-gray-400 truncate" title={filePath}>
                {filePath || '未选择文件'}
              </p>
            </div>

            {/* 快速操作按钮 */}
            <div className="space-y-1">
              <button
                onClick={() => navigate('/sessions')}
                className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-all hover:bg-[#333] text-gray-300 hover:text-white"
              >
                <Settings className="h-4 w-4" />
                会话管理
              </button>
              <button
                onClick={() => navigate('/settings')}
                className="w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm transition-all hover:bg-[#333] text-gray-300 hover:text-white"
              >
                <Settings className="h-4 w-4" />
                API 设置
              </button>
            </div>
          </div>

          {/* 底部调试信息 */}
          {DEBUG && (
            <div className="px-3 py-2 border-t text-xs text-gray-500" style={{ borderColor: '#333' }}>
              调试模式
            </div>
          )}
        </aside>
      )}

      {/* 左侧折叠按钮 */}
      {leftCollapsed && (
        <button
          onClick={() => setLeftCollapsed(false)}
          className="w-8 border-r hover:bg-[#1E1E1E] transition-colors flex items-center justify-center"
          style={{ borderColor: '#333' }}
          title="展开侧边栏"
        >
          <ChevronRight className="h-4 w-4 text-gray-400" />
        </button>
      )}

      {/* ==================== 中心工作区 ==================== */}
      <main className="flex-1 flex flex-col min-w-0 overflow-hidden">
        {/* 顶部标题栏 */}
        <header className="px-6 py-4 border-b" style={{ backgroundColor: '#1E1E1E', borderColor: '#333' }}>
          <div className="flex items-center gap-3">
            <FolderOpen className="h-5 w-5" style={{ color: '#FF6B6B' }} />
            <div className="flex-1 min-w-0">
              <h1 className="text-lg font-semibold text-white truncate">
                Currently Tracking
              </h1>
              <p className="text-xs text-gray-400 truncate mt-0.5">
                {filePath || '未选择会话文件'}
              </p>
            </div>
            <ThemeToggle />
          </div>
        </header>

        {/* 上下分栏：输入区 + 输出区 */}
        <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
          {/* 上半区：Next Goal 输入区 (45%) */}
          <div className="flex flex-col p-6" style={{ height: '45%', backgroundColor: '#121212' }}>
            {/* 珊瑚橙色发光标题 */}
            <h2
              className="text-2xl font-bold mb-4"
              style={{
                color: '#FF6B6B',
                textShadow: '0 0 20px rgba(255, 107, 107, 0.5)'
              }}
            >
              1. NEXT GOAL
            </h2>

            {/* 文件选择和输入区域 */}
            <div className="flex-1 flex flex-col gap-4 min-h-0">
              {/* 文件路径输入 */}
              <div className="flex gap-2">
                <input
                  value={filePath}
                  onChange={(e) => setFilePath(e.target.value)}
                  placeholder="会话文件路径 (.jsonl)"
                  className="flex-1 px-4 py-2 rounded-lg text-sm bg-[#1E1E1E] border border-[#333] text-white placeholder-gray-500 focus:outline-none focus:border-[#FF6B6B] transition-colors"
                />
                <button
                  onClick={autoDetectFile}
                  className="px-4 py-2 text-sm rounded-lg bg-[#333] text-gray-300 hover:bg-[#444] transition-colors whitespace-nowrap"
                >
                  Auto Detect
                </button>
              </div>

              {/* 大型无边框文本输入框 */}
              <textarea
                value={goal}
                onChange={(e) => setGoal(e.target.value)}
                placeholder="在此输入你的下一个目标...&#10;例如：修复用户服务中的空指针异常"
                className="flex-1 min-h-[120px] px-4 py-3 rounded-lg bg-[#1E1E1E] border border-[#333] text-white placeholder-gray-500 focus:outline-none focus:border-[#FF6B6B] transition-colors resize-none"
                style={{ fontSize: '16px', lineHeight: '1.6' }}
              />

              {/* 珊瑚橙色全宽发光按钮 */}
              <button
                onClick={handleAnalyze}
                disabled={analyzing || !goal.trim()}
                className={cn(
                  "w-full py-4 text-white font-semibold rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed",
                  "hover:shadow-lg active:scale-[0.99]"
                )}
                style={{
                  background: 'linear-gradient(135deg, #FF6B6B 0%, #FF8E8E 100%)',
                  boxShadow: '0 0 20px rgba(255, 107, 107, 0.4)'
                }}
                onMouseEnter={(e) => {
                  if (!analyzing && goal.trim()) {
                    e.currentTarget.style.boxShadow = '0 0 30px rgba(255, 107, 107, 0.6)';
                  }
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.boxShadow = '0 0 20px rgba(255, 107, 107, 0.4)';
                }}
              >
                {analyzing ? (
                  <span className="flex items-center justify-center gap-2">
                    <RefreshCw className="h-4 w-4 animate-spin" />
                    分析中...
                  </span>
                ) : "Analyze & Generate Prompt ➔"}
              </button>
            </div>
          </div>

          {/* 分隔线 */}
          <div className="h-px" style={{ backgroundColor: '#333' }}></div>

          {/* 下半区：AI Analysis Result 输出区 (55%) */}
          <div className="flex flex-col p-6 overflow-hidden" style={{ height: '55%', backgroundColor: '#0A0A0A' }}>
            {/* 天空蓝色发光标题 */}
            <h2
              className="text-2xl font-bold mb-4"
              style={{
                color: '#4A9EFF',
                textShadow: '0 0 20px rgba(74, 158, 255, 0.5)'
              }}
            >
              2. AI ANALYSIS RESULT
            </h2>

            {/* 结构化输出画布 */}
            <div className="flex-1 rounded-lg overflow-hidden" style={{ backgroundColor: '#1E1E1E', border: '1px solid #333' }}>
              <div className="h-full overflow-y-auto p-4">
                {analysisResult ? (
                  <pre className="whitespace-pre-wrap break-words text-sm leading-relaxed" style={{ color: '#E0E0E0', fontFamily: 'Consolas, Monaco, "Courier New", monospace' }}>
                    {analysisResult}
                  </pre>
                ) : (
                  <div className="flex items-center justify-center h-full">
                    <p className="text-gray-500">分析结果将显示在这里...</p>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </main>

      {/* ==================== 右侧栏：时间线日志 ==================== */}
      {!rightCollapsed && (
        <aside className="w-[240px] border-l shrink-0 flex flex-col" style={{ backgroundColor: '#1E1E1E', borderColor: '#333' }}>
          {/* 头部 */}
          <div className="flex items-center justify-between px-4 py-3 border-b" style={{ borderColor: '#333' }}>
            <div>
              <h2 className="text-sm font-semibold text-white">时间线日志</h2>
              <p className="text-xs text-gray-400">{timelineLogs.length} 条记录</p>
            </div>
            <div className="flex items-center gap-2">
              {/* 刷新控制 */}
              <div className="flex gap-1">
                <button
                  onClick={() => loadParsedEvents(filePath)}
                  className="p-1.5 rounded hover:bg-[#333] transition-colors"
                  title="刷新"
                  disabled={autoRefresh}
                >
                  <RefreshCw className={cn("h-3.5 w-3.5 text-gray-400", autoRefresh && "opacity-50")} />
                </button>
                <button
                  onClick={toggleAutoRefresh}
                  className={cn(
                    "p-1.5 rounded transition-colors",
                    autoRefresh ? "bg-[#FF6B6B] text-white" : "hover:bg-[#333] text-gray-400"
                  )}
                  title={autoRefresh ? '停止自动刷新' : '开启自动刷新'}
                >
                  {autoRefresh ? '⏸' : '▶'}
                </button>
              </div>
              <button
                onClick={() => setRightCollapsed(true)}
                className="p-1 rounded hover:bg-[#333] transition-colors"
                title="折叠侧边栏"
              >
                <ChevronRight className="h-4 w-4 text-gray-400" />
              </button>
            </div>
          </div>

          {/* 时间线日志列表 */}
          <div className="flex-1 overflow-y-auto p-3 space-y-3">
            {parseError && (
              <div className="p-2 rounded text-xs" style={{ backgroundColor: 'rgba(239, 68, 68, 0.1)', border: '1px solid rgba(239, 68, 68, 0.3)', color: '#FCA5A5' }}>
                {parseError}
              </div>
            )}

            {timelineLogs.length === 0 && !parseError && (
              <div className="text-center py-8">
                <p className="text-sm text-gray-500">暂无日志记录</p>
              </div>
            )}

            {timelineLogs.map((log) => (
              <div
                key={log.id}
                className="p-3 rounded-lg border transition-all hover:shadow-lg"
                style={{
                  backgroundColor: '#121212',
                  borderColor: '#333',
                }}
                onMouseEnter={(e) => {
                  const color = log.type === 'user' ? '255, 107, 107' : '74, 158, 255';
                  e.currentTarget.style.boxShadow = `0 0 20px rgba(${color}, 0.2)`;
                  e.currentTarget.style.borderColor = `rgba(${color}, 0.3)`;
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.boxShadow = 'none';
                  e.currentTarget.style.borderColor = '#333';
                }}
              >
                {/* 顶部：类型图标 + 时间 */}
                <div className="flex items-center gap-2 mb-2">
                  {/* 橙色/蓝色小点 */}
                  <div
                    className="w-2 h-2 rounded-full"
                    style={{
                      backgroundColor: log.type === 'user' ? '#FF6B6B' : '#4A9EFF',
                      boxShadow: `0 0 8px ${log.type === 'user' ? 'rgba(255, 107, 107, 0.5)' : 'rgba(74, 158, 255, 0.5)'}`
                    }}
                  />
                  <span className="text-xs text-gray-400">
                    {new Date(log.timestamp).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                  </span>
                </div>

                {/* 内容摘要 */}
                <p className="text-xs text-gray-300 line-clamp-3" style={{
                  display: '-webkit-box',
                  WebkitLineClamp: 3,
                  WebkitBoxOrient: 'vertical',
                  overflow: 'hidden',
                  lineHeight: '1.5'
                }}>
                  {log.content}
                </p>
              </div>
            ))}
          </div>

          {/* 底部信息 */}
          <div className="px-3 py-2 border-t text-xs text-gray-500 text-center" style={{ borderColor: '#333' }}>
            {autoRefresh && '自动刷新中...'}
          </div>
        </aside>
      )}

      {/* 右侧折叠按钮 */}
      {rightCollapsed && (
        <button
          onClick={() => setRightCollapsed(false)}
          className="w-8 border-l hover:bg-[#1E1E1E] transition-colors flex items-center justify-center"
          style={{ borderColor: '#333' }}
          title="展开时间线"
        >
          <ChevronLeft className="h-4 w-4 text-gray-400" />
        </button>
      )}
    </div>
  );
}

export default App;
