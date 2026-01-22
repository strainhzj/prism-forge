import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { RefreshCw, CheckCircle, AlertCircle, Copy, Check } from "lucide-react";
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "./components/ui/resizable";
import { SideNav } from "./components/navigation";
import { ProjectCard } from "./components/project";
import { TimelineSidebar } from "./components/timeline";
import { ThemeToggle } from "./components/ThemeToggle";
import { LanguageSwitcher } from "./components/LanguageSwitcher";
import {
  useProjectActions,
  useCurrentProject,
  useCurrentSessionFile,
  useProjectLoading,
} from "./stores/useProjectStore";
import { useCurrentSessionActions, useCurrentSessionStore } from "./stores/useCurrentSessionStore";
import { cn } from "@/lib/utils";
import type { EnhancedPrompt } from "@/types/prompt";

// ==================== 类型定义 ====================

type AlertType = 'success' | 'error' | 'info';

interface AlertState {
  show: boolean;
  type: AlertType;
  message: string;
}

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[App] ${action}`, ...args);
  }
}

// ==================== 主组件 ====================

function App() {
  const { t } = useTranslation('index');
  const { t: commonT } = useTranslation('common'); // 获取 common 命名空间的翻译函数
  const navigate = useNavigate();
  const currentProject = useCurrentProject();
  const currentSessionFile = useCurrentSessionFile();
  const { fetchProjects, setCurrentSessionFile, getLatestSessionFile, getSessionFiles } = useProjectActions();
  const projectLoading = useProjectLoading();

  // 获取全局会话状态管理 actions（不解构，避免创建新引用）
  const globalSessionActions = useCurrentSessionActions();

  // 全局 Alert 状态
  const [globalAlert, setGlobalAlert] = useState<AlertState>({
    show: false,
    type: 'info',
    message: '',
  });

  // 显示全局 Alert
  const showGlobalAlert = useCallback((type: AlertType, message: string) => {
    setGlobalAlert({ show: true, type, message });

    // 自动关闭
    const duration = type === 'success' ? 2000 : 3000;
    setTimeout(() => {
      setGlobalAlert(prev => ({ ...prev, show: false }));
    }, duration);
  }, []);

  // 本地状态
  const [goal, setGoal] = useState("");
  const [analysisResult, setAnalysisResult] = useState<EnhancedPrompt | null>(null);
  const [analyzing, setAnalyzing] = useState(false);
  const [rightCollapsed, setRightCollapsed] = useState(false);
  const [copiedPrompt, setCopiedPrompt] = useState(false); // 复制状态

  // 右侧面板 ref，用于编程式控制折叠
  const rightPanelRef = useRef<any>(null);

  // 用于追踪已同步的会话，避免重复同步
  const lastSyncedSessionRef = useRef<string | null>(null);

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
  }, [currentProject]); // 移除 currentSessionFile 依赖，避免无限循环

  // 应用启动时同步会话状态到全局 store
  useEffect(() => {
    if (currentProject && currentSessionFile && currentSessionFile !== lastSyncedSessionRef.current) {
      debugLog('useEffect', 'syncing session to global store', currentSessionFile);
      lastSyncedSessionRef.current = currentSessionFile;

      // 直接使用 store 方法，不依赖 globalSessionActions
      const store = useCurrentSessionStore.getState();
      store.setCurrentSession({
        sessionId: currentSessionFile.split(/[/\\]/).pop() || 'unknown',
        filePath: currentSessionFile,
        projectName: currentProject.name,
      });
    }
  }, [currentProject, currentSessionFile]);

  // 自动检测最新会话文件
  const autoDetectFile = async () => {
    if (!currentProject) return;

    try {
      const path = await getLatestSessionFile(currentProject.path);
      if (path) {
        debugLog('autoDetectFile', 'latest file:', path);
        setCurrentSessionFile(path);

        // 同时设置全局会话状态
        globalSessionActions.setCurrentSession({
          sessionId: path.split(/[/\\]/).pop() || 'unknown',
          filePath: path,
          projectName: currentProject.name,
        });
      } else {
        debugLog('autoDetectFile', 'no files found');
        // 清除全局会话状态
        globalSessionActions.clearCurrentSession();
      }
    } catch (e) {
      const errorMsg = t('messages.autoDetectFileError', { error: String(e) });
      debugLog('autoDetectFile', 'error', errorMsg);
    }
  };

  // 执行分析
  const handleAnalyze = async () => {
    if (!goal) {
      alert(t('alerts.fillGoal', { ns: 'common' }));
      return;
    }

    // 检查是否有当前会话
    if (!currentSessionFile) {
      alert('请先在首页选择一个会话');
      return;
    }

    setAnalyzing(true);
    setAnalysisResult(null);

    try {
      // 使用新的请求结构，传递当前会话文件路径
      const result = await invoke<EnhancedPrompt>("optimize_prompt", {
        request: {
          goal: goal.trim(),
          currentSessionFilePath: currentSessionFile,  // 使用新字段
        }
      });
      debugLog('handleAnalyze', 'result received:', result);
      debugLog('handleAnalyze', 'result JSON:', JSON.stringify(result, null, 2));
      debugLog('handleAnalyze', 'enhancedPrompt length:', result?.enhancedPrompt?.length || 0);
      debugLog('handleAnalyze', 'enhancedPrompt content:', result?.enhancedPrompt || 'EMPTY');
      debugLog('handleAnalyze', 'result keys:', result ? Object.keys(result) : 'NO KEYS');
      setAnalysisResult(result);
    } catch (e) {
      // 更详细的错误处理
      let errorMsg = 'Unknown error';
      if (typeof e === 'string') {
        errorMsg = e;
      } else if (e instanceof Error) {
        errorMsg = e.message;
      } else if (e && typeof e === 'object') {
        try {
          errorMsg = JSON.stringify(e);
        } catch {
          errorMsg = 'Error object cannot be stringified';
        }
      }
      debugLog('handleAnalyze', 'error', e);
      setAnalysisResult(null);
      alert(`Error: ${errorMsg}`);
    } finally {
      setAnalyzing(false);
    }
  };

  // 项目切换确认回调
  const handleProjectChange = useCallback(() => {
    debugLog('handleProjectChange', 'project changed');
    // 移除自动检测，避免覆盖用户选择的会话文件
  }, []);

  // 复制提示词到剪贴板
  const handleCopyPrompt = useCallback(async () => {
    if (!analysisResult?.enhancedPrompt) return;

    try {
      await navigator.clipboard.writeText(analysisResult.enhancedPrompt);
      setCopiedPrompt(true);
      // 2秒后恢复状态
      setTimeout(() => {
        setCopiedPrompt(false);
      }, 2000);
    } catch (error) {
      debugLog('handleCopyPrompt', 'copy failed', error);
    }
  }, [analysisResult?.enhancedPrompt]);

  // 从文件路径提取 sessionId
  // 会话文件名格式：claude-UUID.jsonl，提取 UUID 作为 sessionId
  const sessionId = useMemo(() => {
    if (!currentSessionFile) return '';
    const fileName = currentSessionFile.split(/[\\/]/).pop() || '';
    // 移除 .jsonl 扩展名和 claude- 前缀
    const match = fileName.match(/claude-([^.]+)\.jsonl$/);
    return match ? match[1] : fileName.replace(/\.jsonl$/, '');
  }, [currentSessionFile]);

  return (
    <div className="flex h-screen relative" style={{ fontFamily: 'sans-serif', backgroundColor: 'var(--color-bg-primary)' }}>
      {/* 全局 Alert 弹窗 */}
      {globalAlert.show && (
        <div
          className={cn(
            "fixed top-4 left-1/2 -translate-x-1/2 z-50 max-w-md w-full mx-auto px-4",
            "animate-in fade-in slide-in-from-top-4 duration-300"
          )}
        >
          <div
            className={cn(
              "rounded-lg shadow-lg border px-4 py-3 flex items-center gap-3",
              globalAlert.type === 'success'
                ? "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 border-green-200 dark:border-green-800"
                : globalAlert.type === 'error'
                ? "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300 border-red-200 dark:border-red-800"
                : "bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 border-blue-200 dark:border-blue-800"
            )}
          >
            {globalAlert.type === 'success' && <CheckCircle className="h-5 w-5 flex-shrink-0" />}
            {globalAlert.type === 'error' && <AlertCircle className="h-5 w-5 flex-shrink-0" />}
            <span className="flex-1 text-sm font-medium">{globalAlert.message}</span>
          </div>
        </div>
      )}

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
              <h1 className="text-lg font-bold" style={{ color: 'var(--color-accent-warm)' }}>{t('appTitle')}</h1>
            </div>

          {/* 导航菜单 */}
          <SideNav />

          {/* 底部调试信息 */}
          {DEBUG && (
            <div className="px-3 py-2 mt-auto border-t text-xs" style={{ borderColor: 'var(--color-border-light)', color: 'var(--color-text-secondary)' }}>
              {t('debugMode')}
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
            <header className="px-6 py-4 border-b flex justify-end gap-2" style={{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border-light)' }}>
              {/* 语言切换 */}
              <LanguageSwitcher />
              {/* 主题切换 */}
              <ThemeToggle />
            </header>

            {/* 上下分栏：项目卡片 + 输入区 + 输出区 */}
            <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
              {/* 项目卡片区 (10%) */}
              <div className="p-6" style={{ height: '15%', backgroundColor: 'var(--color-bg-primary)' }}>
                <ProjectCard onConfirm={handleProjectChange} onAlert={showGlobalAlert} />
              </div>

              {/* 分隔线 */}
              <div className="h-px" style={{ backgroundColor: 'var(--color-border-light)' }}></div>

              {/* Next Goal 输入区 (40%) */}
              <div className="flex flex-col p-6" style={{ height: '40%', backgroundColor: 'var(--color-bg-primary)' }}>
                {/* 暖橙色/珊瑚橙色发光标题 */}
                <h2 className="text-2xl font-bold mb-4" style={{ color: 'var(--color-accent-warm)' }}>
                  {t('sections.nextGoal')}
                </h2>

                {/* 目标输入区域 */}
                <div className="flex-1 flex flex-col gap-4 min-h-0">
                  {/* 大型文本输入框 */}
                  <textarea
                    value={goal}
                    onChange={(e) => setGoal(e.target.value)}
                    placeholder={t('placeholders.goalInput')}
                    className="flex-1 min-h-[120px] px-4 py-3 rounded-lg focus:outline-none transition-colors resize-none"
                    style={{
                      fontSize: '16px',
                      lineHeight: '1.6',
                      backgroundColor: 'var(--color-bg-card)',
                      border: '1px solid var(--color-border-light)',
                      color: 'var(--color-text-primary)'
                    }}
                    disabled={projectLoading}
                  />

                  {/* 暖橙色/珊瑚橙色全宽按钮 */}
                  <button
                    onClick={handleAnalyze}
                    disabled={analyzing || !goal.trim()}
                    className={cn(
                      "w-full py-4 text-white font-semibold rounded-lg transition-all disabled:opacity-50 disabled:cursor-not-allowed",
                      "hover:shadow-lg active:scale-[0.99]"
                    )}
                    style={{
                      background: 'var(--color-accent-warm)',
                      boxShadow: '0 0 20px rgba(245, 158, 11, 0.4)'
                    }}
                    onMouseEnter={(e) => {
                      if (!analyzing && goal.trim()) {
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
                        {t('status.analyzing', { ns: 'common' })}
                      </span>
                    ) : t('buttons.analyzeButton')}
                  </button>
                </div>
              </div>

              {/* 分隔线 */}
              <div className="h-px" style={{ backgroundColor: 'var(--color-border-light)' }}></div>

              {/* 下半区：AI Analysis Result 输出区 (50%) */}
              <div className="flex flex-col p-6 overflow-hidden" style={{ height: '50%', backgroundColor: 'var(--color-app-result-bg)' }}>
                {/* 专业蓝色/天空蓝色发光标题 */}
                <h2 className="text-2xl font-bold mb-4" style={{ color: 'var(--color-accent-blue)' }}>
                  {t('sections.analysisResult')}
                </h2>

                {/* 结构化输出画布 */}
                <div className="flex-1 rounded-lg overflow-hidden" style={{
                  backgroundColor: 'var(--color-bg-card)',
                  border: '1px solid var(--color-border-light)'
                }}>
                  <div className="h-full overflow-y-auto p-4">
                    {analysisResult ? (
                      <div className="space-y-4">
                        {/* Token 统计 */}
                        {analysisResult.tokenStats && (
                          <div className="flex items-center gap-4 text-sm" style={{ color: 'var(--color-text-secondary)' }}>
                            <span>Token: {analysisResult.tokenStats.compressedTokens} / {analysisResult.tokenStats.originalTokens}</span>
                            {analysisResult.tokenStats.savingsPercentage > 0 && (
                              <span style={{ color: 'var(--color-accent-blue)' }}>
                                {t('messages.tokenStats.saved')} {analysisResult.tokenStats.savingsPercentage.toFixed(1)}%
                              </span>
                            )}
                            <span>{t('messages.tokenStats.confidence')}: {(analysisResult.confidence * 100).toFixed(0)}%</span>
                          </div>
                        )}

                        {/* 引用的会话 */}
                        {analysisResult.referencedSessions && analysisResult.referencedSessions.length > 0 && (
                          <div className="space-y-2">
                            <p className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>
                              {t('messages.referencedSessions', { count: analysisResult.referencedSessions.length })}
                            </p>
                            {analysisResult.referencedSessions.map((session, idx) => (
                              <div key={idx} className="text-sm p-2 rounded" style={{
                                backgroundColor: 'var(--color-bg-primary)',
                                border: '1px solid var(--color-border-light)'
                              }}>
                                <div className="flex justify-between items-start">
                                  <div className="flex-1">
                                    <p className="font-medium">{session.summary || t('messages.noSummary')}</p>
                                    <p className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                                      {t('messages.similarity')}: {((session.similarityScore || 0) * 100).toFixed(0)}%
                                    </p>
                                  </div>
                                </div>
                              </div>
                            ))}
                          </div>
                        )}

                        {/* 增强的提示词 */}
                        <div>
                          <div className="flex items-center justify-between mb-2">
                            <p className="text-sm font-medium" style={{ color: 'var(--color-text-primary)' }}>
                              {t('messages.enhancedPrompt')}
                            </p>
                            <button
                              onClick={handleCopyPrompt}
                              className={cn(
                                "flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium transition-all",
                                "hover:opacity-80 active:scale-95"
                              )}
                              style={{
                                color: copiedPrompt ? 'var(--color-accent-blue)' : 'var(--color-text-secondary)',
                                backgroundColor: 'var(--color-bg-primary)',
                                border: '1px solid var(--color-border-light)'
                              }}
                              disabled={!analysisResult.enhancedPrompt}
                            >
                              {copiedPrompt ? (
                                <>
                                  <Check className="h-3.5 w-3.5" />
                                  {commonT('buttons.copied')}
                                </>
                              ) : (
                                <>
                                  <Copy className="h-3.5 w-3.5" />
                                  {commonT('buttons.copy')}
                                </>
                              )}
                            </button>
                          </div>
                          <pre className="whitespace-pre-wrap break-words text-sm leading-relaxed p-3 rounded" style={{
                            color: 'var(--color-text-primary)',
                            fontFamily: 'Consolas, Monaco, "Courier New", monospace',
                            backgroundColor: 'var(--color-bg-primary)',
                            border: '1px solid var(--color-border-light)'
                          }}>
                            {analysisResult.enhancedPrompt}
                          </pre>
                        </div>
                      </div>
                    ) : (
                      <div className="flex items-center justify-center h-full">
                        <p style={{ color: 'var(--color-text-secondary)' }}>
                          {analyzing ? t('messages.analyzing') : t('messages.analysisResultPlaceholder')}
                        </p>
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
            sessionId={sessionId}
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
