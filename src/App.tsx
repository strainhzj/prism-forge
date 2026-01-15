import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { RefreshCw } from "lucide-react";
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "./components/ui/resizable";
import { SideNav } from "./components/navigation";
import { ProjectCard } from "./components/project";
import { TimelineSidebar } from "./components/timeline";
import { ThemeToggle } from "./components/ThemeToggle";
import {
  useProjectActions,
  useCurrentProject,
  useCurrentSessionFile,
  useProjectLoading,
} from "./stores/useProjectStore";
import { cn } from "@/lib/utils";

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[App] ${action}`, ...args);
  }
}

// ==================== Layout Persistence ====================

const LAYOUT_STORAGE_KEY = 'prism-forge-layout';

interface LayoutSizes {
  leftPanel: number; // 像素值
  rightPanel: number; // 像素值
}

const DEFAULT_LAYOUT: LayoutSizes = {
  leftPanel: 200, // 左侧导航栏宽度 (像素)
  rightPanel: 240, // 右侧时间线宽度 (像素)
};

/**
 * 从 localStorage 加载布局配置
 */
function loadLayoutSizes(): LayoutSizes {
  try {
    const saved = localStorage.getItem(LAYOUT_STORAGE_KEY);
    if (saved) {
      const parsed = JSON.parse(saved) as LayoutSizes;
      debugLog('loadLayoutSizes', '从 localStorage 加载布局:', parsed);
      return parsed;
    }
  } catch (e) {
    console.error('加载布局配置失败:', e);
  }
  return DEFAULT_LAYOUT;
}

/**
 * 保存布局配置到 localStorage
 */
function saveLayoutSizes(sizes: LayoutSizes) {
  try {
    localStorage.setItem(LAYOUT_STORAGE_KEY, JSON.stringify(sizes));
    debugLog('saveLayoutSizes', '保存布局到 localStorage:', sizes);
  } catch (e) {
    console.error('保存布局配置失败:', e);
  }
}

// ==================== 主组件 ====================

function App() {
  const navigate = useNavigate();
  const currentProject = useCurrentProject();
  const currentSessionFile = useCurrentSessionFile();
  const { fetchProjects, setCurrentSessionFile } = useProjectActions();
  const projectLoading = useProjectLoading();

  // 本地状态
  const [goal, setGoal] = useState("");
  const [analysisResult, setAnalysisResult] = useState("");
  const [analyzing, setAnalyzing] = useState(false);
  const [rightCollapsed, setRightCollapsed] = useState(false);

  // 右侧面板 ref，用于编程式控制折叠
  const rightPanelRef = useRef<any>(null);

  // 切换右侧面板折叠
  const toggleRightCollapse = useCallback(() => {
    if (rightCollapsed) {
      // 展开：调用 expand 方法
      rightPanelRef.current?.expand();
    } else {
      // 折叠：调用 collapse 方法
      rightPanelRef.current?.collapse();
    }
    setRightCollapsed(!rightCollapsed);
  }, [rightCollapsed]);

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

  // 初始化：加载项目列表
  useEffect(() => {
    debugLog('useEffect', 'initializing, fetching projects');
    fetchProjects();
  }, [fetchProjects]);

  // 项目切换后自动检测最新文件
  useEffect(() => {
    if (currentProject && !currentSessionFile) {
      debugLog('useEffect', 'project changed, auto detecting latest file');
      autoDetectFile();
    }
  }, [currentProject, currentSessionFile]);

  // 自动检测最新会话文件
  const autoDetectFile = async () => {
    if (!currentProject) return;

    try {
      const path = await invoke<string>("get_latest_session_path");
      debugLog('autoDetectFile', 'latest file:', path);
      setCurrentSessionFile(path);
    } catch (e) {
      const errorMsg = `自动检测文件失败: ${e}`;
      debugLog('autoDetectFile', 'error', errorMsg);
    }
  };

  // 执行分析
  const handleAnalyze = async () => {
    if (!currentSessionFile || !goal) {
      alert("请填写目标内容");
      return;
    }

    setAnalyzing(true);
    setAnalysisResult("");

    try {
      const result = await invoke<string>("optimize_prompt", {
        sessionFile: currentSessionFile,
        goal
      });
      setAnalysisResult(result);
    } catch (e) {
      setAnalysisResult(`Error: ${e}`);
    } finally {
      setAnalyzing(false);
    }
  };

  // 项目切换确认回调
  const handleProjectChange = useCallback(() => {
    debugLog('handleProjectChange', 'project changed, reloading file');
    autoDetectFile();
  }, [currentProject]);

  return (
    <div className="flex h-screen" style={{ fontFamily: 'sans-serif', backgroundColor: 'var(--color-bg-primary)' }}>
      <ResizablePanelGroup
        orientation="horizontal"
        className="h-full"
      >
        {/* ==================== 左侧导航栏 ==================== */}
        <ResizablePanel
          id="left-panel"
          defaultSize={200}
          minSize={150}
        >
          <div
            className="flex flex-col h-full"
            style={{ backgroundColor: 'var(--color-bg-card)' }}
          >
            {/* Logo/标题 */}
            <div className="px-4 py-4 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
              <h1 className="text-lg font-bold" style={{ color: 'var(--color-accent-warm)' }}>PrismForge</h1>
            </div>

          {/* 导航菜单 */}
          <SideNav />

          {/* 底部调试信息 */}
          {DEBUG && (
            <div className="px-3 py-2 mt-auto border-t text-xs" style={{ borderColor: 'var(--color-border-light)', color: 'var(--color-text-secondary)' }}>
              调试模式
            </div>
          )}
          </div>
        </ResizablePanel>

        {/* 左侧拖动条 */}
        <ResizableHandle />

        {/* ==================== 中心工作区 ==================== */}
        <ResizablePanel defaultSize={600}>
          <main className="flex-1 flex flex-col min-w-0 overflow-hidden h-full">
            {/* 顶部标题栏 */}
            <header className="px-6 py-4 border-b flex justify-end" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
              {/* 主题切换 */}
              <ThemeToggle />
            </header>

            {/* 上下分栏：项目卡片 + 输入区 + 输出区 */}
            <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
              {/* 项目卡片区 (10%) */}
              <div className="p-6" style={{ height: '15%', backgroundColor: 'var(--color-bg-primary)' }}>
                <ProjectCard onConfirm={handleProjectChange} />
              </div>

              {/* 分隔线 */}
              <div className="h-px" style={{ backgroundColor: 'var(--color-border-light)' }}></div>

              {/* Next Goal 输入区 (40%) */}
              <div className="flex flex-col p-6" style={{ height: '40%', backgroundColor: 'var(--color-bg-primary)' }}>
                {/* 暖橙色/珊瑚橙色发光标题 */}
                <h2 className="text-2xl font-bold mb-4" style={{ color: 'var(--color-accent-warm)' }}>
                  1. 下一步目标
                </h2>

                {/* 目标输入区域 */}
                <div className="flex-1 flex flex-col gap-4 min-h-0">
                  {/* 大型文本输入框 */}
                  <textarea
                    value={goal}
                    onChange={(e) => setGoal(e.target.value)}
                    placeholder="在此输入你的下一个目标...&#10;例如：修复用户服务中的空指针异常"
                    className="flex-1 min-h-[120px] px-4 py-3 rounded-lg focus:outline-none transition-colors resize-none"
                    style={{
                      fontSize: '16px',
                      lineHeight: '1.6',
                      backgroundColor: 'var(--color-bg-card)',
                      border: '1px solid var(--color-border-light)',
                      color: 'var(--color-text-primary)'
                    }}
                    disabled={projectLoading || !currentSessionFile}
                  />

                  {/* 暖橙色/珊瑚橙色全宽按钮 */}
                  <button
                    onClick={handleAnalyze}
                    disabled={analyzing || !goal.trim() || !currentSessionFile}
                    className={cn(
                      "w-full py-4 text-white font-semibold rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed",
                      "hover:shadow-lg active:scale-[0.99]"
                    )}
                    style={{
                      background: 'var(--color-accent-warm)',
                      boxShadow: '0 0 20px rgba(245, 158, 11, 0.4)'
                    }}
                    onMouseEnter={(e) => {
                      if (!analyzing && goal.trim() && currentSessionFile) {
                        e.currentTarget.style.boxShadow = '0 0 30px rgba(245, 158, 11, 0.6)';
                      }
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.boxShadow = '0 0 20px rgba(245, 158, 11, 0.4)';
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
              <div className="h-px" style={{ backgroundColor: 'var(--color-border-light)' }}></div>

              {/* 下半区：AI Analysis Result 输出区 (50%) */}
              <div className="flex flex-col p-6 overflow-hidden" style={{ height: '50%', backgroundColor: 'var(--color-app-result-bg)' }}>
                {/* 专业蓝色/天空蓝色发光标题 */}
                <h2 className="text-2xl font-bold mb-4" style={{ color: 'var(--color-accent-blue)' }}>
                  2. AI 分析结果
                </h2>

                {/* 结构化输出画布 */}
                <div className="flex-1 rounded-lg overflow-hidden" style={{
                  backgroundColor: 'var(--color-bg-card)',
                  border: '1px solid var(--color-border-light)'
                }}>
                  <div className="h-full overflow-y-auto p-4">
                    {analysisResult ? (
                      <pre className="whitespace-pre-wrap break-words text-sm leading-relaxed" style={{
                        color: 'var(--color-text-primary)',
                        fontFamily: 'Consolas, Monaco, "Courier New", monospace'
                      }}>
                        {analysisResult}
                      </pre>
                    ) : (
                      <div className="flex items-center justify-center h-full">
                        <p style={{ color: 'var(--color-text-secondary)' }}>分析结果将显示在这里...</p>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </div>
          </main>
        </ResizablePanel>

        {/* 右侧拖动条 */}
        <ResizableHandle />

        {/* ==================== 右侧栏：时间线日志 ==================== */}
        <ResizablePanel
          id="right-panel"
          defaultSize={250}
          minSize={200}
          collapsible={true}
          collapsedSize={32}  // 折叠后仅显示展开按钮的宽度
          panelRef={rightPanelRef}
        >
          <TimelineSidebar
            filePath={currentSessionFile || ''}
            autoRefreshInterval={3000}
            className="h-full"
            collapsed={rightCollapsed}
            onToggleCollapse={toggleRightCollapse}
          />
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

export default App;
