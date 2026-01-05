/**
 * 导出功能相关类型定义
 *
 * 用于会话数据导出为不同格式
 */

/**
 * 导出格式
 */
export type ExportFormat = 'json' | 'csv' | 'markdown';

/**
 * 导出选项
 */
export interface ExportOptions {
  /**
   * 导出格式
   */
  format: ExportFormat;
  /**
   * 是否包含元数据
   */
  includeMetadata?: boolean;
  /**
   * 是否包含代码块
   */
  includeCodeBlocks?: boolean;
  /**
   * 是否包含时间戳
   */
  includeTimestamps?: boolean;
  /**
   * CSV 分隔符
   */
  csvDelimiter?: ',' | ';' | '\t';
  /**
   * Markdown 标题级别
   */
  markdownHeadingLevel?: 1 | 2 | 3;
}

/**
 * 导出数据项（消息）
 */
export interface ExportDataItem {
  /**
   * 时间戳
   */
  timestamp?: string;
  /**
   * 角色
   */
  role: 'user' | 'assistant' | 'system';
  /**
   * 内容
   */
  content: string;
  /**
   * 代码块（可选）
   */
  codeBlocks?: Array<{
    language: string;
    code: string;
  }>;
  /**
   * 元数据（可选）
   */
  metadata?: Record<string, unknown>;
}

/**
 * 导出数据（会话）
 */
export interface ExportData {
  /**
   * 会话 ID
   */
  sessionId: string;
  /**
   * 会话标题
   */
  title?: string;
  /**
   * 项目路径
   */
  projectPath?: string;
  /**
   * 创建时间
   */
  createdAt?: string;
  /**
   * 更新时间
   */
  updatedAt?: string;
  /**
   * 消息列表
   */
  messages: ExportDataItem[];
  /**
   * 统计信息
   */
  stats?: {
    totalMessages: number;
    totalTokens?: number;
    codeChanges?: number;
  };
}

/**
 * 导出结果
 */
export interface ExportResult {
  /**
   * 文件名
   */
  filename: string;
  /**
   * 文件内容（Blob URL）
   */
  content: string;
  /**
   * 文件大小（字节）
   */
  size: number;
  /**
   * MIME 类型
   */
  mimeType: string;
}

/**
 * 导出错误
 */
export interface ExportError {
  /**
   * 错误代码
   */
  code: string;
  /**
   * 错误消息
   */
  message: string;
  /**
   * 错误详情
   */
  details?: unknown;
}
