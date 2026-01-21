/**
 * TimelineSidebar 组件
 *
 * 右侧时间线日志侧边栏，从 App.tsx 提取
 * 集成多级日志读取功能
 */

import { useState, useCallback, useEffect, useMemo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { ChevronRight, ChevronLeft, RefreshCw, Filter, Check } from 'lucide-react';
import { cn } from '@/lib/utils';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useViewLevelManager, useSessionContent } from '@/hooks/useViewLevel';
import { VIEW_LEVEL_INFO, AVAILABLE_VIEW_LEVELS } from '@/types/viewLevel';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[TimelineSidebar] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface ParsedEvent {
  time: string;
  role: string;
  content: string;
  fullContent: string;  // 完整内容用于详情弹窗
  event_type: string;
}

export interface TimelineLog {
  id: string;
  timestamp: string;
  type: 'user' | 'assistant' | 'system';
  content: string;
  fullContent: string; // 完整内容用于弹窗
  tooltipContent: string; // 截断内容用于 tooltip
}

export interface TimelineSidebarProps {
  /**
   * 当前会话文件路径
   */
  filePath: string;
  /**
   * 会话 ID
   */
  sessionId: string;
  /**
   * 自动刷新间隔（毫秒），0 表示不自动刷新
   */
  autoRefreshInterval?: number;
  /**
   * 自定义类名
   */
  className?: string;
  /**
   * 是否折叠
   */
  collapsed?: boolean;
  /**
   * 切换折叠状态的回调
   */
  onToggleCollapse?: () => void;
}

// 自动刷新间隔（毫秒）
const DEFAULT_AUTO_REFRESH_INTERVAL = 3000;

// Tooltip 内容最大长度
const TOOLTIP_MAX_LENGTH = 500;

// 下拉框固定宽度
const DROPDOWN_WIDTH = 140;

/**
 * 从内容中提取文本（extracted 模式）
 * 参考 TimelineMessageList 的 extractTextFromContent 函数
 *
 * @param content - 原始内容（JSON 字符串）
 * @param isUser - 是否是用户消息
 * @returns 提取后的文本内容
 */
function extractTextFromContent(content: string, isUser: boolean): string {
  if (!content) return '';

  try {
    const parsed = JSON.parse(content);

    // extracted 模式：从 JSON 中提取内容
    if (typeof parsed === 'object' && parsed !== null) {
      // 用户消息或助手消息：提取 content 字段
      if ('content' in parsed) {
        const msgContent = parsed.content;

        // 如果 content 是数组，提取所有 text 字段
        if (Array.isArray(msgContent)) {
          const texts = msgContent
            .map((item: unknown) => {
              if (typeof item === 'object' && item !== null && 'text' in item) {
                return String((item as { text: unknown }).text);
              }
              return null;
            })
            .filter((text): text is string => text !== null);
          return texts.join('\n\n');
        }

        // 如果 content 是字符串，直接返回
        if (typeof msgContent === 'string') {
          return msgContent;
        }

        // 如果 content 是其他类型，尝试转字符串
        return String(msgContent);
      }

      // 兼容：如果有顶级 text 字段，返回 text（主要针对助手消息）
      if (!isUser && 'text' in parsed) {
        return String(parsed.text);
      }
    }

    // 如果找不到对应字段，返回格式化的原始内容
    return content;
  } catch {
    // 解析失败，返回原始内容
    return content;
  }
}

/**
 * 格式化文本内容
 * 将 `\n` 转换为真正的换行符
 *
 * @param text - 文本内容
 * @returns 格式化后的文本
 */
function formatTextContent(text: string): string {
  if (!text) return '';

  // 将 \n 转换为真正的换行符
  return text.replace(/\\n/g, '\n');
}

/**
 * 日志详情对话框组件
 * 提取为独立组件以避免代码重复
 */
interface LogDetailDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  log: TimelineLog | null;
}

function LogDetailDialog({ open, onOpenChange, log }: LogDetailDialogProps) {
  const { t } = useTranslation('index');

  if (!log) return null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>{t('timeline.logDetail')}</DialogTitle>
        </DialogHeader>
        <div className="flex-1 overflow-y-auto">
          {/* 元信息 */}
          <div className="flex items-center gap-2 mb-4 pb-3 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
            <div
              className="w-2 h-2 rounded-full"
              style={{
                backgroundColor:
                  log.type === 'user' ? 'var(--color-accent-warm)' : 'var(--color-accent-blue)',
                boxShadow:
                  log.type === 'user'
                    ? '0 0 8px rgba(245, 158, 11, 0.5)'
                    : '0 0 8px rgba(37, 99, 235, 0.5)',
              }}
            />
            <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
              {log.type === 'user' ? t('timeline.user') : t('timeline.assistant')}
            </span>
            <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
              {new Date(log.timestamp).toLocaleString('zh-CN', {
                year: 'numeric',
                month: '2-digit',
                day: '2-digit',
                hour: '2-digit',
                minute: '2-digit',
                second: '2-digit',
              })}
            </span>
          </div>

          {/* 完整内容 */}
          <pre
            className="text-sm whitespace-pre-wrap break-words leading-relaxed"
            style={{
              color: 'var(--color-text-primary)',
              fontFamily: 'Consolas, Monaco, "Courier New", monospace',
              fontSize: '13px',
              lineHeight: '1.6',
            }}
          >
            {formatJsonContent(log.fullContent)}
          </pre>
        </div>
      </DialogContent>
    </Dialog>
  );
}

/**
 * 尝试格式化 JSON 内容
 * 如果内容是有效的 JSON，返回格式化后的字符串
 * 否则返回原内容
 */
function formatJsonContent(content: string): string {
  const trimmed = content.trim();

  // 尝试检测并提取 JSON 部分
  // 情况1: 整个内容就是 JSON
  if (trimmed.startsWith('{') || trimmed.startsWith('[')) {
    try {
      const parsed = JSON.parse(trimmed);
      return JSON.stringify(parsed, null, 2);
    } catch {
      // 不是有效 JSON，继续检查
    }
  }

  // 情况2: 内容中包含 JSON（例如 "完整输出:\n{...}"）
  // 尝试找到第一个 { 或 [ 的位置
  const jsonStartIndex = trimmed.indexOf('{') !== -1 ? trimmed.indexOf('{')
    : trimmed.indexOf('[') !== -1 ? trimmed.indexOf('[')
    : -1;

  if (jsonStartIndex > 0) {
    const potentialJson = trimmed.slice(jsonStartIndex);
    try {
      const parsed = JSON.parse(potentialJson);
      const formattedJson = JSON.stringify(parsed, null, 2);
      // 保留前缀文本 + 格式化的 JSON
      return trimmed.slice(0, jsonStartIndex) + '\n' + formattedJson;
    } catch {
      // 不是有效 JSON
    }
  }

  // 无法格式化，返回原内容
  return content;
}

/**
 * TimelineSidebar 组件
 *
 * @example
 * <TimelineSidebar filePath="/path/to/session.jsonl" sessionId="xxx" />
 */
export function TimelineSidebar({
  filePath,
  sessionId,
  autoRefreshInterval = DEFAULT_AUTO_REFRESH_INTERVAL,
  className,
  collapsed = false,
  onToggleCollapse,
}: TimelineSidebarProps) {
  const { t } = useTranslation('index');
  const { t: tSessions } = useTranslation('sessions');
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedLog, setSelectedLog] = useState<TimelineLog | null>(null);
  const [viewLevelDropdownOpen, setViewLevelDropdownOpen] = useState(false);
  const [dropdownAlign, setDropdownAlign] = useState<'left' | 'right'>('left');
  const viewLevelDropdownRef = useRef<HTMLDivElement>(null);

  // ===== 多级日志读取功能 =====
  // 使用视图等级管理 hook，默认使用问答对模式
  const {
    currentViewLevel,
    changeViewLevel,
    isSaving: viewLevelSaving
  } = useViewLevelManager(sessionId);

  // 加载会话内容（根据视图等级过滤）
  // 注意：禁用 useSessionContent 的自动刷新，使用组件自己的自动刷新逻辑
  const {
    messages,
    isLoading: contentLoading,
    error: contentError,
    refresh: refreshContent,
  } = useSessionContent(sessionId, currentViewLevel, filePath, false);

  // 自动刷新处理函数
  const handleAutoRefresh = useCallback(async () => {
    if (autoRefresh) {
      debugLog('auto-refresh', '自动刷新中...');
      await refreshContent();
    }
  }, [autoRefresh, refreshContent]);

  // 初始加载
  useEffect(() => {
    void refreshContent();
  }, [filePath, refreshContent]);

  // 自动刷新定时器
  useEffect(() => {
    if (autoRefresh && filePath && autoRefreshInterval > 0) {
      debugLog('auto-refresh', '启动自动刷新，间隔:', autoRefreshInterval);
      const intervalId = setInterval(() => {
        void handleAutoRefresh();
      }, autoRefreshInterval);

      return () => {
        debugLog('auto-refresh', '清除自动刷新定时器');
        clearInterval(intervalId);
      };
    }
  }, [autoRefresh, filePath, autoRefreshInterval, handleAutoRefresh]);

  // 将消息转换为时间线日志格式
  const timelineLogs: TimelineLog[] = useMemo(() => {
    if (!messages || messages.length === 0) return [];

    return messages.map((msg, i) => {
      const rawContent = msg.summary || '';
      const msgTypeLower = msg.msgType?.toLowerCase() || '';
      const isUser = msgTypeLower === 'user';

      // 提取 text 字段内容
      const extractedText = extractTextFromContent(rawContent, isUser);
      // 格式化文本（将 \n 转换为真正的换行符）
      const formattedText = formatTextContent(extractedText);

      const displayContent = formattedText.length > 200
        ? formattedText.substring(0, 200) + '...'
        : formattedText;
      const tooltipContent = formattedText.length > TOOLTIP_MAX_LENGTH
        ? formattedText.substring(0, TOOLTIP_MAX_LENGTH) + '...'
        : formattedText;

      // 根据消息类型判断日志类型
      let logType: 'user' | 'assistant' | 'system' = 'system';
      if (msgTypeLower === 'user') {
        logType = 'user';
      } else if (msgTypeLower === 'assistant') {
        logType = 'assistant';
      }

      return {
        id: `log-${i}`,
        timestamp: msg.timestamp || new Date().toISOString(),
        type: logType,
        content: displayContent,
        fullContent: formattedText, // 使用提取和格式化后的内容
        tooltipContent,
      };
    });
  }, [messages]);

  // 计算下拉框对齐方式
  useEffect(() => {
    if (viewLevelDropdownOpen && viewLevelDropdownRef.current) {
      const rect = viewLevelDropdownRef.current.getBoundingClientRect();
      const viewportWidth = window.innerWidth;
      const spaceOnRight = viewportWidth - rect.right;

      // 如果右侧空间不足，则向左对齐
      if (spaceOnRight < DROPDOWN_WIDTH) {
        setDropdownAlign('right');
      } else {
        setDropdownAlign('left');
      }
    }
  }, [viewLevelDropdownOpen]);

  // 点击外部关闭下拉框
  const handleClickOutside = useCallback((event: MouseEvent) => {
    if (viewLevelDropdownRef.current && !viewLevelDropdownRef.current.contains(event.target as Node)) {
      setViewLevelDropdownOpen(false);
    }
  }, []);

  useEffect(() => {
    if (viewLevelDropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
    }
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [viewLevelDropdownOpen, handleClickOutside]);

  // 获取当前视图等级的显示名称
  const currentViewLevelLabel = useMemo(() => {
    return VIEW_LEVEL_INFO[currentViewLevel]?.displayName || currentViewLevel;
  }, [currentViewLevel]);

  // 切换自动刷新开关
  const toggleAutoRefresh = useCallback(() => {
    setAutoRefresh((prev) => !prev);
  }, []);

  if (collapsed) {
    // 折叠状态 - 显示展开按钮
    return (
      <>
        <div className="h-full flex flex-col" style={{ backgroundColor: 'var(--color-bg-card)' }}>
          <button
            onClick={() => onToggleCollapse?.()}
            className="flex-1 w-8 border-l transition-colors flex items-center justify-center hover:bg-[var(--color-bg-card)]"
            style={{ borderColor: 'var(--color-border-light)' }}
            title={t('timeline.expand')}
          >
            <ChevronLeft className="h-4 w-4" style={{ color: 'var(--color-text-secondary)' }} />
          </button>
        </div>
        {/* 折叠状态下也保留对话框 */}
        <LogDetailDialog
          open={dialogOpen}
          onOpenChange={setDialogOpen}
          log={selectedLog}
        />
      </>
    );
  }

  return (
    <>
      <aside
        className={cn('border-l shrink-0 flex flex-col', className)}
        style={{
          backgroundColor: 'var(--color-bg-card)',
          borderColor: 'var(--color-border-light)',
        }}
      >
      {/* 头部 */}
      <div
        className="flex flex-col gap-3 px-4 py-3 border-b"
        style={{ borderColor: 'var(--color-border-light)' }}
      >
        {/* 第一行：标题和记录数 */}
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-sm font-semibold" style={{ color: 'var(--color-text-primary)' }}>
              {t('timeline.title')}
            </h2>
            <p className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
              {timelineLogs.length} {t('timeline.recordCount')}
            </p>
          </div>
          <button
            onClick={() => onToggleCollapse?.()}
            className="p-1 rounded transition-colors hover:bg-[var(--color-app-secondary)]"
            title={t('timeline.collapse')}
          >
            <ChevronRight className="h-4 w-4" style={{ color: 'var(--color-text-secondary)' }} />
          </button>
        </div>

        {/* 第二行：视图等级 + 刷新按钮 */}
        <div className="flex items-center justify-end gap-1">
          {/* 视图等级按钮 */}
          <div className="relative" ref={viewLevelDropdownRef}>
            <button
              onClick={() => setViewLevelDropdownOpen(prev => !prev)}
              disabled={viewLevelSaving || contentLoading}
              className="p-1.5 rounded transition-colors hover:bg-[var(--color-app-secondary)] disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1"
              title={`${tSessions('viewLevel.title')}: ${currentViewLevelLabel}`}
            >
              <Filter className="h-3.5 w-3.5" style={{ color: 'var(--color-text-secondary)' }} />
            </button>

            {/* 下拉菜单 */}
            {viewLevelDropdownOpen && (
              <div
                className={cn(
                  "absolute top-full mt-1 z-50 rounded-lg shadow-lg py-1",
                  dropdownAlign === 'left' ? 'left-0' : 'right-0'
                )}
                style={{
                  width: DROPDOWN_WIDTH,
                  backgroundColor: 'var(--color-bg-card)',
                  border: '1px solid var(--color-border-light)',
                }}
              >
                {AVAILABLE_VIEW_LEVELS.map((level) => {
                  const isSelected = level === currentViewLevel;
                  const levelInfo = VIEW_LEVEL_INFO[level];
                  return (
                    <button
                      key={level}
                      onClick={() => {
                        changeViewLevel(level);
                        setViewLevelDropdownOpen(false);
                      }}
                      disabled={viewLevelSaving || contentLoading}
                      className={cn(
                        'w-full text-left px-3 py-2 text-xs transition-colors flex items-center gap-2',
                        'hover:bg-[var(--color-app-secondary)]',
                        'disabled:opacity-50 disabled:cursor-not-allowed'
                      )}
                      style={{
                        color: isSelected ? 'var(--color-accent-warm)' : 'var(--color-text-primary)',
                      }}
                    >
                      <span className="text-base">{levelInfo?.icon}</span>
                      <span className="flex-1">{levelInfo?.displayName}</span>
                      {isSelected && <Check className="h-3 w-3" style={{ color: 'var(--color-accent-warm)' }} />}
                    </button>
                  );
                })}
              </div>
            )}
          </div>

          {/* 刷新按钮（自动刷新开关） */}
          <button
            onClick={toggleAutoRefresh}
            disabled={contentLoading}
            className={cn(
              'p-1.5 rounded transition-colors',
              autoRefresh ? 'text-white' : ''
            )}
            style={
              autoRefresh
                ? { backgroundColor: 'var(--color-accent-warm)' }
                : { color: 'var(--color-text-secondary)' }
            }
            onMouseEnter={(e) => {
              if (!autoRefresh) {
                e.currentTarget.style.backgroundColor = 'var(--color-app-secondary)';
              }
            }}
            onMouseLeave={(e) => {
              if (!autoRefresh) {
                e.currentTarget.style.backgroundColor = 'transparent';
              }
            }}
            title={autoRefresh ? t('timeline.stopAutoRefresh') : t('timeline.startAutoRefresh')}
          >
            <RefreshCw className={cn('h-3.5 w-3.5', autoRefresh && 'animate-spin')} />
          </button>
        </div>
      </div>

      {/* 时间线日志列表 */}
      <div className="flex-1 overflow-y-auto p-3 space-y-3">
        {contentError && (
          <div
            className="p-2 rounded text-xs"
            style={{
              backgroundColor: 'var(--color-app-error-bg)',
              border: '1px solid var(--color-app-error-border)',
              color: 'var(--color-app-error-text)',
            }}
          >
            {String(contentError)}
          </div>
        )}

        {contentLoading && timelineLogs.length === 0 && (
          <div className="text-center py-8">
            <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
              加载中...
            </p>
          </div>
        )}

        {!contentLoading && timelineLogs.length === 0 && (
          <div className="text-center py-8">
            <p className="text-sm" style={{ color: 'var(--color-text-secondary)' }}>
              {t('timeline.noLogs')}
            </p>
          </div>
        )}

        {timelineLogs.map((log) => (
          <div
            key={log.id}
            className="p-3 rounded-lg border transition-all hover:shadow-lg"
            style={{
              backgroundColor: 'var(--color-bg-primary)',
              borderColor: 'var(--color-border-light)',
            }}
            onMouseEnter={(e) => {
              const isUser = log.type === 'user';
              const color = isUser ? '245, 158, 11' : '37, 99, 235'; // warm orange or blue
              e.currentTarget.style.boxShadow = `0 0 20px rgba(${color}, 0.2)`;
              e.currentTarget.style.borderColor = `rgba(${color}, 0.3)`;
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.boxShadow = 'none';
              e.currentTarget.style.borderColor = 'var(--color-border-light)';
            }}
          >
            {/* 顶部：类型图标 + 时间 */}
            <div className="flex items-center gap-2 mb-2">
              {/* 暖橙色/蓝色小点 */}
              <div
                className="w-2 h-2 rounded-full"
                style={{
                  backgroundColor:
                    log.type === 'user' ? 'var(--color-accent-warm)' : 'var(--color-accent-blue)',
                  boxShadow:
                    log.type === 'user'
                      ? '0 0 8px rgba(245, 158, 11, 0.5)'
                      : '0 0 8px rgba(37, 99, 235, 0.5)',
                }}
              />
              <span className="text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                {new Date(log.timestamp).toLocaleTimeString('zh-CN', {
                  hour: '2-digit',
                  minute: '2-digit',
                  second: '2-digit',
                })}
              </span>
            </div>

            {/* 内容摘要 */}
            <p
              className="text-xs line-clamp-3"
              title={log.tooltipContent}
              onClick={() => {
                setSelectedLog(log);
                setDialogOpen(true);
              }}
              style={{
                color: 'var(--color-text-primary)',
                display: '-webkit-box',
                WebkitLineClamp: 3,
                WebkitBoxOrient: 'vertical',
                overflow: 'hidden',
                lineHeight: '1.5',
                cursor: 'pointer', // 鼠标悬停时显示指针
              }}
            >
              {log.content}
            </p>
          </div>
        ))}
      </div>

      {/* 底部信息 */}
      <div
        className="px-3 py-2 border-t text-xs text-center flex items-center justify-center gap-2"
        style={{
          borderColor: 'var(--color-border-light)',
          color: 'var(--color-text-secondary)',
        }}
      >
        {/* 显示当前视图等级 */}
        <span>{currentViewLevelLabel}</span>
        {autoRefresh && <span>· {t('timeline.autoRefreshing')}</span>}
      </div>
    </aside>

    {/* 详情对话框 */}
    <LogDetailDialog
      open={dialogOpen}
      onOpenChange={setDialogOpen}
      log={selectedLog}
    />
    </>
  );
}
