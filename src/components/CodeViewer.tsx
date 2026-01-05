/**
 * CodeViewer 组件
 *
 * 专门用于在消息中展示代码块的便捷组件
 * 基于 MonacoEditor，提供预设的配置和样式
 */

import { useState, useCallback, useMemo } from 'react';
import { Check, Copy, Maximize2, Minimize2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { MonacoEditor, MonacoEditorUtils } from './MonacoEditor';

export interface CodeViewerProps {
  /**
   * 代码内容
   */
  code: string;
  /**
   * 编程语言
   */
  language?: string;
  /**
   * 是否显示复制按钮（默认 true）
   */
  showCopyButton?: boolean;
  /**
   * 是否显示语言标识（默认 true）
   */
  showLanguage?: boolean;
  /**
   * 是否支持全屏切换（默认 true）
   */
  allowFullscreen?: boolean;
  /**
   * 初始高度（默认 300px）
   */
  height?: string | number;
  /**
   * 最小高度
   */
  minHeight?: string | number;
  /**
   * 最大高度
   */
  maxHeight?: string | number;
  /**
   * 是否自适应高度（默认 true）
   */
  autoHeight?: boolean;
  /**
   * 自定义类名
   */
  className?: string;
  /**
   * 文件名（用于推断语言）
   */
  filename?: string;
  /**
   * 主题（默认 vs-dark）
   */
  theme?: 'vs-dark' | 'light' | 'vs';
}

/**
 * 计算代码行数
 */
function countLines(code: string): number {
  if (!code) return 0;
  return code.split('\n').length;
}

/**
 * 根据代码行数计算建议高度
 */
function calculateSuggestedHeight(lineCount: number): number {
  // 每行约 20px，加上上下边距
  const minHeight = 150;
  const maxHeight = 600;
  const lineHeight = 20;
  const padding = 40;

  const suggestedHeight = Math.max(minHeight, Math.min(maxHeight, lineCount * lineHeight + padding));
  return suggestedHeight;
}

/**
 * CodeViewer 组件
 */
export function CodeViewer({
  code,
  language: propLanguage,
  showCopyButton = true,
  showLanguage = true,
  allowFullscreen = true,
  height = 'auto',
  minHeight = 150,
  maxHeight = 600,
  autoHeight = true,
  className,
  filename,
  theme = 'vs-dark',
}: CodeViewerProps) {
  const [copied, setCopied] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);

  /**
   * 推断编程语言
   */
  const language = useMemo(() => {
    if (propLanguage) return propLanguage;
    if (filename) return MonacoEditorUtils.inferLanguageFromExtension(filename);
    return 'typescript';
  }, [propLanguage, filename]);

  /**
   * 计算高度
   */
  const calculatedHeight = useMemo(() => {
    if (!autoHeight || typeof height === 'string') return height;

    const lineCount = countLines(code);
    const suggestedHeight = calculateSuggestedHeight(lineCount);
    return Math.min(Number(height) || suggestedHeight, suggestedHeight);
  }, [code, height, autoHeight]);

  /**
   * 复制代码
   */
  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('复制失败:', error);
    }
  }, [code]);

  /**
   * 切换全屏
   */
  const toggleFullscreen = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  /**
   * 获取显示语言名称
   */
  const displayLanguage = useMemo(() => {
    const languageDisplayNames: Record<string, string> = {
      typescript: 'TypeScript',
      javascript: 'JavaScript',
      jsx: 'JSX',
      tsx: 'TSX',
      python: 'Python',
      rust: 'Rust',
      go: 'Go',
      java: 'Java',
      csharp: 'C#',
      cpp: 'C++',
      html: 'HTML',
      css: 'CSS',
      json: 'JSON',
      yaml: 'YAML',
      markdown: 'Markdown',
      sql: 'SQL',
      bash: 'Bash',
      powershell: 'PowerShell',
    };
    return languageDisplayNames[language] || language;
  }, [language]);

  return (
    <div
      className={cn(
        'group relative my-4 rounded-lg border bg-background overflow-hidden',
        isFullscreen && 'fixed inset-0 z-50 rounded-none',
        className
      )}
    >
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-4 py-2 border-b bg-muted/50">
        <div className="flex items-center gap-2">
          {showLanguage && (
            <>
              <span className="text-xs font-medium text-muted-foreground uppercase">
                {displayLanguage}
              </span>
              {filename && (
                <>
                  <span className="text-muted-foreground/50">•</span>
                  <span className="text-xs text-muted-foreground truncate max-w-[200px]">
                    {filename}
                  </span>
                </>
              )}
            </>
          )}
        </div>

        <div className="flex items-center gap-1">
          {allowFullscreen && (
            <Button
              variant="ghost"
              size="sm"
              onClick={toggleFullscreen}
              className="h-7 px-2 opacity-0 group-hover:opacity-100 transition-opacity"
              title={isFullscreen ? '退出全屏' : '全屏'}
            >
              {isFullscreen ? (
                <Minimize2 className="h-4 w-4" />
              ) : (
                <Maximize2 className="h-4 w-4" />
              )}
            </Button>
          )}

          {showCopyButton && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCopy}
              className="h-7 px-2 opacity-0 group-hover:opacity-100 transition-opacity"
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

      {/* 代码编辑器 */}
      <div
        className={cn(
          'overflow-auto',
          isFullscreen && 'h-[calc(100%-52px)]'
        )}
        style={
          isFullscreen
            ? {}
            : {
                height: autoHeight ? calculatedHeight : height,
                minHeight: !autoHeight ? minHeight : undefined,
                maxHeight: !autoHeight ? maxHeight : undefined,
              }
        }
      >
        <MonacoEditor
          value={code}
          language={language}
          theme={theme}
          readOnly={true}
          height="100%"
          fontSize={13}
          minimap={isFullscreen}
          lineNumbers="on"
          folding={true}
          automaticLayout={true}
          className="border-0"
        />
      </div>
    </div>
  );
}

/**
 * 内联代码查看器（适用于单行代码）
 */
export interface InlineCodeViewerProps {
  children: string;
  className?: string;
}

export function InlineCodeViewer({ children, className }: InlineCodeViewerProps) {
  return (
    <code
      className={cn(
        'px-1.5 py-0.5 rounded bg-muted text-sm font-mono text-foreground',
        className
      )}
    >
      {children}
    </code>
  );
}

/**
 * 代码块预览（适用于卡片式展示）
 */
export interface CodePreviewProps {
  code: string;
  language?: string;
  filename?: string;
  maxHeight?: string | number;
  className?: string;
}

export function CodePreview({
  code,
  language = 'typescript',
  filename,
  maxHeight = 400,
  className,
}: CodePreviewProps) {
  return (
    <CodeViewer
      code={code}
      language={language}
      filename={filename}
      height={maxHeight}
      showCopyButton={true}
      showLanguage={true}
      allowFullscreen={true}
      autoHeight={false}
      className={cn('shadow-sm', className)}
    />
  );
}
