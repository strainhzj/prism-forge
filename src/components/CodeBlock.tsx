/**
 * CodeBlock 组件
 *
 * 代码块显示，支持语法高亮和复制功能
 */

import { useState, useCallback, useMemo } from 'react';
import { Check, Copy } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';

export interface CodeBlockProps {
  /**
   * 代码内容
   */
  code: string;
  /**
   * 编程语言（用于语法高亮）
   */
  language?: string;
  /**
   * 是否显示复制按钮
   */
  showCopyButton?: boolean;
  /**
   * 是否显示语言标识
   */
  showLanguage?: boolean;
  /**
   * 最大高度（超过则滚动）
   */
  maxHeight?: string;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 简单的语法高亮（基于关键词和模式匹配）
 */
function highlightCode(code: string, _language?: string): string {
  // 转义 HTML
  let escaped = code
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // 关键词高亮
  const keywords = ['const', 'let', 'var', 'function', 'return', 'if', 'else', 'for', 'while', 'class', 'import', 'export', 'from', 'async', 'await', 'try', 'catch', 'throw', 'new', 'this', 'true', 'false', 'null', 'undefined'];

  keywords.forEach((keyword) => {
    const regex = new RegExp(`\\b${keyword}\\b`, 'g');
    escaped = escaped.replace(regex, `<span class="text-purple-600 dark:text-purple-400 font-semibold">${keyword}</span>`);
  });

  // 字符串高亮
  escaped = escaped.replace(/(".*?"|'.*?'|`.*?`)/g, '<span class="text-green-600 dark:text-green-400">$1</span>');

  return escaped;
}

/**
 * CodeBlock 组件
 */
export function CodeBlock({
  code,
  language,
  showCopyButton = true,
  showLanguage = true,
  maxHeight,
  className,
}: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('复制失败:', error);
    }
  }, [code]);

  const highlightedCode = useMemo(() => {
    return highlightCode(code, language);
  }, [code, language]);

  return (
    <div className={cn('group relative my-4 rounded-lg border bg-muted/50 overflow-hidden', className)}>
      <div className="flex items-center justify-between px-4 py-2 border-b bg-muted">
        <div className="flex items-center gap-2">
          {showLanguage && language && (
            <span className="text-xs font-medium text-muted-foreground uppercase">
              {language}
            </span>
          )}
        </div>

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
                已复制
              </>
            ) : (
              <>
                <Copy className="h-4 w-4 mr-1" />
                复制
              </>
            )}
          </Button>
        )}
      </div>

      <pre
        className={cn(
          'p-4 overflow-x-auto text-sm font-mono',
          maxHeight && `max-h-[${maxHeight}] overflow-y-auto`
        )}
      >
        <code
          dangerouslySetInnerHTML={{
            __html: highlightedCode,
          }}
        />
      </pre>
    </div>
  );
}

/**
 * 内联代码组件
 */
export interface InlineCodeProps {
  children: string;
  className?: string;
}

export function InlineCode({ children, className }: InlineCodeProps) {
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
