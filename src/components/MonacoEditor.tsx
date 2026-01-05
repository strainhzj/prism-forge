/**
 * MonacoEditor 组件
 *
 * 封装 Monaco Editor，提供功能强大的代码编辑器/查看器
 * 支持 TypeScript 语法高亮、深色主题、行号、只读模式
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import Editor, { Monaco, OnMount } from '@monaco-editor/react';
import { cn } from '@/lib/utils';

export interface MonacoEditorProps {
  /**
   * 代码内容
   */
  value: string;
  /**
   * 编程语言（默认 typescript）
   */
  language?: string;
  /**
   * 主题（默认 vs-dark）
   */
  theme?: 'vs-dark' | 'light' | 'vs';
  /**
   * 是否只读模式（默认 true）
   */
  readOnly?: boolean;
  /**
   * 是否显示行号（默认 true）
   */
  lineNumbers?: 'on' | 'off' | 'relative' | 'interval';
  /**
   * 是否显示小地图（默认 true）
   */
  minimap?: boolean;
  /**
   * 字体大小（默认 14）
   */
  fontSize?: number;
  /**
   * 是否自动布局（默认 true）
   */
  automaticLayout?: boolean;
  /**
   * 高度（默认 100%）
   */
  height?: string | number;
  /**
   * 最小高度（可选）
   */
  minHeight?: string | number;
  /**
   * 最大高度（可选）
   */
  maxHeight?: string | number;
  /**
   * 内容变化回调
   */
  onChange?: (value: string | undefined) => void;
  /**
   * 自定义类名
   */
  className?: string;
  /**
   * 编辑器挂载回调
   */
  onMount?: (editor: any, monaco: Monaco) => void;
  /**
   * 加载前回调（可用于配置 Monaco）
   */
  beforeMount?: (monaco: Monaco) => void;
  /**
   * 是否显示滚动条（默认 true）
   */
  scrollbar?: {
    vertical?: 'auto' | 'visible' | 'hidden';
    horizontal?: 'auto' | 'visible' | 'hidden';
  };
  /**
   * 代码折叠（默认 true）
   */
  folding?: boolean;
  /**
   * 括号配对高亮（默认 true）
   */
  bracketPairColorization?: boolean;
  /**
   * 是否启用代码 Lens（默认 false）
   */
  codeLens?: boolean;
  /**
   * 是否显示上下文菜单（默认 true）
   */
  contextmenu?: boolean;
}

/**
 * 默认编辑器选项
 */
const defaultOptions = {
  readOnly: true,
  lineNumbers: 'on' as const,
  minimap: { enabled: true },
  fontSize: 14,
  automaticLayout: true,
  scrollbar: {
    vertical: 'auto' as const,
    horizontal: 'auto' as const,
  },
  folding: true,
  bracketPairColorization: {
    enabled: true,
  },
  codeLens: false,
  contextmenu: true,
  renderLineHighlight: 'all' as const,
  cursorBlinking: 'smooth' as const,
  cursorSmoothCaretAnimation: 'on' as const,
  smoothScrolling: true,
  tabSize: 2,
  wordWrap: 'off' as const,
};

/**
 * MonacoEditor 组件
 */
export function MonacoEditor({
  value,
  language = 'typescript',
  theme = 'vs-dark',
  readOnly = true,
  lineNumbers = 'on',
  minimap = true,
  fontSize = 14,
  automaticLayout = true,
  height = '100%',
  minHeight,
  maxHeight,
  onChange,
  className,
  onMount,
  beforeMount,
  scrollbar,
  folding = true,
  bracketPairColorization = true,
  codeLens = false,
  contextmenu = true,
}: MonacoEditorProps) {
  const [isLoading, setIsLoading] = useState(true);
  const editorRef = useRef<any>(null);

  /**
   * 编辑器挂载处理
   */
  const handleEditorDidMount: OnMount = useCallback((editor, monaco) => {
    editorRef.current = editor;
    setIsLoading(false);

    // 调用自定义挂载回调
    if (onMount) {
      onMount(editor, monaco);
    }
  }, [onMount]);

  /**
   * Monaco 挂载前配置
   */
  const handleBeforeMount = useCallback((monaco: Monaco) => {
    // 配置 TypeScript 选项
    monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
      noSemanticValidation: false,
      noSyntaxValidation: false,
    });

    monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
      target: monaco.languages.typescript.ScriptTarget.ES2020,
      allowNonTsExtensions: true,
      moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
      module: monaco.languages.typescript.ModuleKind.CommonJS,
      noEmit: true,
      esModuleInterop: true,
      jsx: monaco.languages.typescript.JsxEmit.React,
      reactNamespace: 'React',
      allowJs: true,
      typeRoots: ['node_modules/@types'],
    });

    // 调用自定义配置回调
    if (beforeMount) {
      beforeMount(monaco);
    }
  }, [beforeMount]);

  /**
   * 清理资源
   */
  useEffect(() => {
    return () => {
      if (editorRef.current) {
        editorRef.current.dispose();
      }
    };
  }, []);

  /**
   * 构建编辑器选项
   */
  const options = {
    ...defaultOptions,
    readOnly,
    lineNumbers,
    minimap: { enabled: minimap },
    fontSize,
    automaticLayout,
    scrollbar: scrollbar || defaultOptions.scrollbar,
    folding,
    bracketPairColorization: {
      enabled: bracketPairColorization,
    },
    codeLens,
    contextmenu,
  };

  // 计算高度样式
  const heightStyle = useMemoHeight(height, minHeight, maxHeight);

  return (
    <div
      className={cn('relative overflow-hidden rounded-lg border bg-background', className)}
      style={heightStyle}
    >
      {isLoading && (
        <div className="absolute inset-0 flex items-center justify-center bg-muted/20 z-10">
          <div className="text-sm text-muted-foreground">加载编辑器...</div>
        </div>
      )}
      <Editor
        height="100%"
        language={language}
        theme={theme}
        value={value}
        options={options}
        onMount={handleEditorDidMount}
        beforeMount={handleBeforeMount}
        onChange={onChange}
        loading={<div className="flex items-center justify-center h-full">加载中...</div>}
      />
    </div>
  );
}

/**
 * 计算高度样式
 */
function useMemoHeight(height: string | number, minHeight?: string | number, maxHeight?: string | number) {
  const style: React.CSSProperties = {
    height: typeof height === 'number' ? `${height}px` : height,
  };

  if (minHeight) {
    style.minHeight = typeof minHeight === 'number' ? `${minHeight}px` : minHeight;
  }

  if (maxHeight) {
    style.maxHeight = typeof maxHeight === 'number' ? `${maxHeight}px` : maxHeight;
  }

  return style;
}

/**
 * 导出编辑器引用类型
 */
export type MonacoEditorRef = ReturnType<typeof Editor.prototype>;

/**
 * 导出配置工具函数
 */
export const MonacoEditorUtils = {
  /**
   * 获取支持的编程语言列表
   */
  getSupportedLanguages: () => [
    'typescript',
    'javascript',
    'jsx',
    'tsx',
    'python',
    'rust',
    'go',
    'java',
    'csharp',
    'cpp',
    'html',
    'css',
    'json',
    'yaml',
    'markdown',
    'sql',
    'bash',
    'powershell',
  ],

  /**
   * 从文件扩展名推断语言
   */
  inferLanguageFromExtension: (filename: string): string => {
    const ext = filename.split('.').pop()?.toLowerCase();
    const languageMap: Record<string, string> = {
      'ts': 'typescript',
      'tsx': 'typescript',
      'js': 'javascript',
      'jsx': 'javascript',
      'py': 'python',
      'rs': 'rust',
      'go': 'go',
      'java': 'java',
      'cs': 'csharp',
      'cpp': 'cpp',
      'c': 'c',
      'html': 'html',
      'css': 'css',
      'json': 'json',
      'yaml': 'yaml',
      'yml': 'yaml',
      'md': 'markdown',
      'sql': 'sql',
      'sh': 'bash',
      'ps1': 'powershell',
    };
    return languageMap[ext || ''] || 'typescript';
  },
};
