/**
 * CodeBlockRenderer 组件
 *
 * 智能渲染消息内容中的代码块和文本
 * 自动检测 Markdown 风格的 ``` 代码块并使用 CodeViewer 渲染
 */

import { useMemo } from 'react';
import { cn } from '@/lib/utils';
import { CodeViewer } from './CodeViewer';
import { parseCodeBlocks } from '@/utils/codeParser';

export interface CodeBlockRendererProps {
  /**
   * 要渲染的内容（可能包含代码块）
   */
  content: string;
  /**
   * 主题（默认 vs-dark）
   */
  theme?: 'vs-dark' | 'light' | 'vs';
  /**
   * 代码块最大高度（默认 600px）
   */
  maxHeight?: string | number;
  /**
   * 是否显示复制按钮（默认 true）
   */
  showCopyButton?: boolean;
  /**
   * 是否支持全屏切换（默认 true）
   */
  allowFullscreen?: boolean;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 渲染文本内容
 */
function TextContent({ content, className }: { content: string; className?: string }) {
  // 处理内联代码 `code`
  const parts = content.split(/(`[^`]+`)/g);

  return (
    <div className={cn('text-sm whitespace-pre-wrap break-words', className)}>
      {parts.map((part, index) => {
        // 检查是否是内联代码
        if (part.startsWith('`') && part.endsWith('`')) {
          const code = part.slice(1, -1);
          return (
            <code
              key={index}
              className="px-1.5 py-0.5 rounded bg-muted text-sm font-mono text-foreground"
            >
              {code}
            </code>
          );
        }

        // 普通文本
        return <span key={index}>{part}</span>;
      })}
    </div>
  );
}

/**
 * CodeBlockRenderer 组件
 */
export function CodeBlockRenderer({
  content,
  theme = 'vs-dark',
  maxHeight = 600,
  showCopyButton = true,
  allowFullscreen = true,
  className,
}: CodeBlockRendererProps) {
  /**
   * 解析内容中的代码块
   */
  const fragments = useMemo(() => {
    return parseCodeBlocks(content);
  }, [content]);

  /**
   * 判断是否只包含单个代码块
   */
  const isSingleCodeBlock = useMemo(() => {
    return fragments.length === 1 && fragments[0].type === 'code';
  }, [fragments]);

  /**
   * 如果只包含单个代码块，直接返回 CodeViewer
   */
  if (isSingleCodeBlock) {
    const fragment = fragments[0] as { type: 'code'; content: { code: string; language: string } };
    return (
      <CodeViewer
        code={fragment.content.code}
        language={fragment.content.language}
        theme={theme}
        maxHeight={maxHeight}
        showCopyButton={showCopyButton}
        allowFullscreen={allowFullscreen}
        className={className}
      />
    );
  }

  /**
   * 渲染混合内容（文本 + 代码块）
   */
  return (
    <div className={cn('space-y-3', className)}>
      {fragments.map((fragment, index) => {
        if (fragment.type === 'text') {
          return (
            <TextContent
              key={`text-${index}`}
              content={fragment.content}
            />
          );
        }

        // 代码块
        const codeBlock = fragment.content;
        return (
          <CodeViewer
            key={`code-${index}`}
            code={codeBlock.code}
            language={codeBlock.language}
            theme={theme}
            maxHeight={maxHeight}
            showCopyButton={showCopyButton}
            allowFullscreen={allowFullscreen}
          />
        );
      })}
    </div>
  );
}

/**
 * 快捷组件：纯代码块渲染（已知内容是代码块时使用）
 */
export interface PureCodeBlockProps {
  code: string;
  language?: string;
  filename?: string;
  theme?: 'vs-dark' | 'light' | 'vs';
  className?: string;
}

export function PureCodeBlock({
  code,
  language = 'typescript',
  filename,
  theme = 'vs-dark',
  className,
}: PureCodeBlockProps) {
  return (
    <CodeViewer
      code={code}
      language={language}
      filename={filename}
      theme={theme}
      className={className}
    />
  );
}
