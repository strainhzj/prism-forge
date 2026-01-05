/**
 * DiffViewer 组件
 *
 * 代码差异对比视图组件
 * 支持并排模式、统一模式切换，语法高亮，大文件性能优化
 */

import { useState, useCallback, useMemo } from 'react';
import { Check, Copy, Columns3, AlignLeft } from 'lucide-react';
import DiffViewerLib from 'react-diff-viewer-continued';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import type { CodeChangeInfo } from '@/types/message';

export interface DiffViewerProps {
  /**
   * 变更前的代码
   */
  oldValue: string;
  /**
   * 变更后的代码
   */
  newValue: string;
  /**
   * 代码变更信息（可选，用于显示文件路径等）
   */
  changeInfo?: CodeChangeInfo;
  /**
   * 是否显示复制按钮（默认 true）
   */
  showCopyButton?: boolean;
  /**
   * 是否显示文件路径（默认 true）
   */
  showFilePath?: boolean;
  /**
   * 初始视图模式（默认 split）
   */
  defaultViewMode?: 'split' | 'unified';
  /**
   * 是否允许切换视图模式（默认 true）
   */
  allowSwitchMode?: boolean;
  /**
   * 最大行数（超过则启用优化模式，默认 1000）
   */
  maxLines?: number;
  /**
   * 自定义类名
   */
  className?: string;
  /**
   * 主题（默认 vs-dark，与 Monaco Editor 一致）
   */
  theme?: 'light' | 'dark';
}

/**
 * 视图模式类型
 */
type ViewMode = 'split' | 'unified';

/**
 * DiffViewer 组件
 */
export function DiffViewer({
  oldValue,
  newValue,
  changeInfo,
  showCopyButton = true,
  showFilePath = true,
  defaultViewMode = 'split',
  allowSwitchMode = true,
  maxLines = 1000,
  className,
  theme = 'dark',
}: DiffViewerProps) {
  const [viewMode, setViewMode] = useState<ViewMode>(defaultViewMode);
  const [copied, setCopied] = useState(false);

  /**
   * 切换视图模式
   */
  const toggleViewMode = useCallback(() => {
    setViewMode((prev) => (prev === 'split' ? 'unified' : 'split'));
  }, []);

  /**
   * 复制新代码
   */
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(newValue);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('复制失败:', error);
    }
  }, [newValue]);

  /**
   * 检查是否为大文件
   */
  const isLargeFile = useMemo(() => {
    const oldLines = oldValue.split('\n').length;
    const newLines = newValue.split('\n').length;
    return Math.max(oldLines, newLines) > maxLines;
  }, [oldValue, newValue, maxLines]);

  /**
   * 计算文件名显示
   */
  const displayFileName = useMemo(() => {
    if (!changeInfo?.file_path) return null;
    const path = changeInfo.file_path;
    // 只显示文件名和最后两级目录
    const parts = path.split('/');
    if (parts.length <= 2) return path;
    return '.../' + parts.slice(-2).join('/');
  }, [changeInfo]);

  /**
   * 获取变更类型标签
   */
  const changeTypeLabel = useMemo(() => {
    if (!changeInfo) return null;
    const labels: Record<string, string> = {
      create: '新增',
      update: '修改',
      delete: '删除',
    };
    return labels[changeInfo.change_type];
  }, [changeInfo]);

  /**
   * 优化大文件渲染：截断显示
   */
  const optimizedOldValue = useMemo(() => {
    if (!isLargeFile) return oldValue;
    const lines = oldValue.split('\n');
    if (lines.length <= maxLines) return oldValue;
    // 保留前后各 300 行
    const keepLines = 300;
    const truncated = [
      ...lines.slice(0, keepLines),
      `... (省略 ${lines.length - keepLines * 2} 行，共 ${lines.length} 行)`,
      ...lines.slice(-keepLines),
    ];
    return truncated.join('\n');
  }, [oldValue, isLargeFile, maxLines]);

  const optimizedNewValue = useMemo(() => {
    if (!isLargeFile) return newValue;
    const lines = newValue.split('\n');
    if (lines.length <= maxLines) return newValue;
    const keepLines = 300;
    const truncated = [
      ...lines.slice(0, keepLines),
      `... (省略 ${lines.length - keepLines * 2} 行，共 ${lines.length} 行)`,
      ...lines.slice(-keepLines),
    ];
    return truncated.join('\n');
  }, [newValue, isLargeFile, maxLines]);

  /**
   * 计算统计信息
   */
  const stats = useMemo(() => {
    const oldLines = oldValue.split('\n').length;
    const newLines = newValue.split('\n').length;
    const linesAdded = changeInfo?.lines_added || 0;
    const linesRemoved = changeInfo?.lines_removed || 0;

    return {
      oldLines,
      newLines,
      linesAdded,
      linesRemoved,
    };
  }, [oldValue, newValue, changeInfo]);

  return (
    <div className={cn('relative my-4 rounded-lg border bg-background overflow-hidden', className)}>
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b bg-muted/50">
        <div className="flex items-center gap-3 flex-1 min-w-0">
          {showFilePath && displayFileName && (
            <>
              <span className="text-sm font-medium text-foreground truncate">
                {displayFileName}
              </span>
              {changeTypeLabel && (
                <>
                  <span className="text-muted-foreground/50">•</span>
                  <span
                    className={cn(
                      'text-xs px-2 py-0.5 rounded',
                      changeInfo?.change_type === 'create' && 'bg-green-500/10 text-green-500',
                      changeInfo?.change_type === 'update' && 'bg-blue-500/10 text-blue-500',
                      changeInfo?.change_type === 'delete' && 'bg-red-500/10 text-red-500'
                    )}
                  >
                    {changeTypeLabel}
                  </span>
                </>
              )}
            </>
          )}

          {/* 统计信息 */}
          {stats.linesAdded > 0 || stats.linesRemoved > 0 ? (
            <>
              <span className="text-muted-foreground/50">•</span>
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                {stats.linesRemoved > 0 && (
                  <span className="text-red-500">-{stats.linesRemoved}</span>
                )}
                {stats.linesAdded > 0 && (
                  <span className="text-green-500">+{stats.linesAdded}</span>
                )}
                {isLargeFile && (
                  <span className="text-orange-500">
                    (共 {Math.max(stats.oldLines, stats.newLines)} 行)
                  </span>
                )}
              </div>
            </>
          ) : null}
        </div>

        <div className="flex items-center gap-1">
          {/* 切换视图模式 */}
          {allowSwitchMode && (
            <Button
              variant="ghost"
              size="sm"
              onClick={toggleViewMode}
              className="h-7 px-2"
              title={viewMode === 'split' ? '切换到统一视图' : '切换到并排视图'}
            >
              {viewMode === 'split' ? (
                <Columns3 className="h-4 w-4" />
              ) : (
                <AlignLeft className="h-4 w-4" />
              )}
              <span className="ml-1 text-xs">
                {viewMode === 'split' ? '并排' : '统一'}
              </span>
            </Button>
          )}

          {/* 复制按钮 */}
          {showCopyButton && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCopy}
              className="h-7 px-2"
            >
              {copied ? (
                <>
                  <Check className="h-4 w-4 mr-1 text-green-500" />
                  <span className="text-xs">已复制</span>
                </>
              ) : (
                <>
                  <Copy className="h-4 w-4 mr-1" />
                  <span className="text-xs">复制</span>
                </>
              )}
            </Button>
          )}
        </div>
      </div>

      {/* Diff 视图 */}
      <div className={cn('overflow-auto', isLargeFile && 'max-h-[600px]')}>
        <DiffViewerLib
          oldValue={optimizedOldValue}
          newValue={optimizedNewValue}
          splitView={viewMode === 'split'}
          useDarkTheme={theme === 'dark'}
          hideLineNumbers={false}
          showDiffOnly={false}
        />
      </div>

      {/* 大文件提示 */}
      {isLargeFile && (
        <div className="px-4 py-2 bg-orange-500/10 border-t border-orange-500/20">
          <p className="text-xs text-orange-500">
            ⚠️ 这是一个大文件（{Math.max(stats.oldLines, stats.newLines)} 行），为优化性能仅显示部分内容
          </p>
        </div>
      )}
    </div>
  );
}

/**
 * 从代码变更信息创建 Diff Viewer
 */
export interface CodeChangeDiffViewerProps {
  changeInfo: CodeChangeInfo;
  viewMode?: 'split' | 'unified';
  allowSwitchMode?: boolean;
  className?: string;
  theme?: 'light' | 'dark';
}

export function CodeChangeDiffViewer({
  changeInfo,
  viewMode = 'split',
  allowSwitchMode = true,
  className,
  theme = 'dark',
}: CodeChangeDiffViewerProps) {
  const oldValue = changeInfo.old_text || '';
  const newValue = changeInfo.new_text || '';

  // 如果没有变更内容，显示提示
  if (!oldValue && !newValue) {
    return (
      <div className={cn('my-4 p-4 rounded-lg border bg-muted/30 text-center text-muted-foreground', className)}>
        <p className="text-sm">无法显示代码差异：缺少变更内容</p>
        {changeInfo.file_path && (
          <p className="text-xs mt-1">文件: {changeInfo.file_path}</p>
        )}
      </div>
    );
  }

  return (
    <DiffViewer
      oldValue={oldValue}
      newValue={newValue}
      changeInfo={changeInfo}
      defaultViewMode={viewMode}
      allowSwitchMode={allowSwitchMode}
      className={className}
      theme={theme}
    />
  );
}

/**
 * 批量代码变更 Diff Viewer
 */
export interface MultiChangeDiffViewerProps {
  changes: CodeChangeInfo[];
  viewMode?: 'split' | 'unified';
  allowSwitchMode?: boolean;
  className?: string;
  theme?: 'light' | 'dark';
}

export function MultiChangeDiffViewer({
  changes,
  viewMode = 'split',
  allowSwitchMode = true,
  className,
  theme = 'dark',
}: MultiChangeDiffViewerProps) {
  if (changes.length === 0) {
    return (
      <div className={cn('my-4 p-4 rounded-lg border bg-muted/30 text-center text-muted-foreground', className)}>
        <p className="text-sm">没有代码变更</p>
      </div>
    );
  }

  return (
    <div className={cn('space-y-4', className)}>
      {changes.map((change, index) => (
        <CodeChangeDiffViewer
          key={`${change.file_path}-${index}`}
          changeInfo={change}
          viewMode={viewMode}
          allowSwitchMode={allowSwitchMode}
          theme={theme}
        />
      ))}
    </div>
  );
}
