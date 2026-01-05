/**
 * 代码块解析工具
 *
 * 从消息内容中提取 Markdown 风格的代码块
 */

export interface CodeBlock {
  /**
   * 代码内容
   */
  code: string;
  /**
   * 编程语言
   */
  language: string;
  /**
   * 代码块在原文中的起始索引
   */
  startIndex: number;
  /**
   * 代码块在原文中的结束索引
   */
  endIndex: number;
}

export interface ParsedContentText {
  /**
   * 文本片段（非代码块）
   */
  type: 'text';
  /**
   * 内容
   */
  content: string;
}

export interface ParsedContentCode {
  /**
   * 代码块片段
   */
  type: 'code';
  /**
   * 代码块数据
   */
  content: CodeBlock;
}

export type ParsedContent = ParsedContentText | ParsedContentCode;

/**
 * 从文本中提取代码块
 *
 * 支持以下格式：
 * - ```language
 *   code
 *   ```
 * - `    code`（4 空格缩进的代码块，仅当前后都是空行时）
 *
 * @param text 要解析的文本
 * @returns 解析后的片段数组
 */
export function parseCodeBlocks(text: string): ParsedContent[] {
  if (!text) return [];

  const fragments: ParsedContent[] = [];
  let currentIndex = 0;

  // 匹配 ```language\ncode\n``` 格式的代码块
  const codeBlockRegex = /```(\w*)\n([\s\S]*?)```/g;
  let match: RegExpExecArray | null;

  while ((match = codeBlockRegex.exec(text)) !== null) {
    // 添加代码块之前的文本
    if (match.index > currentIndex) {
      const textContent = text.slice(currentIndex, match.index);
      if (textContent.trim()) {
        fragments.push({
          type: 'text',
          content: textContent,
        });
      }
    }

    // 添加代码块
    const language = match[1] || 'typescript';
    const code = match[2];
    fragments.push({
      type: 'code',
      content: {
        code,
        language,
        startIndex: match.index,
        endIndex: match.index + match[0].length,
      },
    });

    currentIndex = match.index + match[0].length;
  }

  // 添加剩余的文本
  if (currentIndex < text.length) {
    const remainingText = text.slice(currentIndex);
    if (remainingText.trim()) {
      fragments.push({
        type: 'text',
        content: remainingText,
      });
    }
  }

  // 如果没有找到代码块，返回整个文本作为单个片段
  if (fragments.length === 0 && text.trim()) {
    return [{
      type: 'text',
      content: text,
    }];
  }

  return fragments;
}

/**
 * 检测文本中是否包含代码块
 */
export function hasCodeBlocks(text: string): boolean {
  return /```(\w*)\n[\s\S]*?```/.test(text);
}

/**
 * 从文件路径推断编程语言
 *
 * @param filename 文件名或路径
 * @returns 编程语言标识
 */
export function inferLanguageFromPath(filename: string): string {
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
    'cc': 'cpp',
    'cxx': 'cpp',
    'h': 'c',
    'hpp': 'cpp',
    'hxx': 'cpp',
    'html': 'html',
    'htm': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'sass',
    'less': 'less',
    'json': 'json',
    'yaml': 'yaml',
    'yml': 'yaml',
    'md': 'markdown',
    'markdown': 'markdown',
    'sql': 'sql',
    'sh': 'bash',
    'bash': 'bash',
    'ps1': 'powershell',
    'xml': 'xml',
    'toml': 'toml',
    'ini': 'ini',
    'dockerfile': 'dockerfile',
    'docker': 'dockerfile',
    'rb': 'ruby',
    'php': 'php',
    'swift': 'swift',
    'kt': 'kotlin',
    'dart': 'dart',
    'lua': 'lua',
    'r': 'r',
    'scala': 'scala',
    'ex': 'elixir',
    'exs': 'elixir',
    'erl': 'erlang',
    'hs': 'haskell',
    'clj': 'clojure',
    'fs': 'fsharp',
    'vue': 'vue',
    'svelte': 'svelte',
  };
  return languageMap[ext || ''] || 'typescript';
}

/**
 * 获取语言显示名称
 *
 * @param language 编程语言标识
 * @returns 本地化的显示名称
 */
export function getLanguageDisplayName(language: string): string {
  const displayNames: Record<string, string> = {
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
    c: 'C',
    html: 'HTML',
    css: 'CSS',
    scss: 'SCSS',
    sass: 'Sass',
    less: 'Less',
    json: 'JSON',
    yaml: 'YAML',
    yml: 'YAML',
    markdown: 'Markdown',
    md: 'Markdown',
    sql: 'SQL',
    bash: 'Bash',
    sh: 'Shell',
    powershell: 'PowerShell',
    xml: 'XML',
    toml: 'TOML',
    ini: 'INI',
    dockerfile: 'Dockerfile',
    ruby: 'Ruby',
    php: 'PHP',
    swift: 'Swift',
    kotlin: 'Kotlin',
    dart: 'Dart',
    lua: 'Lua',
    r: 'R',
    scala: 'Scala',
    elixir: 'Elixir',
    erlang: 'Erlang',
    haskell: 'Haskell',
    clojure: 'Clojure',
    fsharp: 'F#',
    vue: 'Vue',
    svelte: 'Svelte',
  };
  return displayNames[language] || language;
}
