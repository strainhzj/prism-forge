/**
 * 导出工具函数
 *
 * 支持将会话数据导出为 JSON、CSV、Markdown 格式
 */

import type {
  ExportData,
  ExportFormat,
  ExportOptions,
  ExportResult
} from '@/types/export';

/**
 * 生成文件名
 */
function generateFilename(
  sessionId: string,
  format: ExportFormat,
  title?: string
): string {
  const timestamp = new Date().toISOString().slice(0, 10);
  const baseTitle = title?.replace(/[^a-zA-Z0-9\u4e00-\u9fa5]/g, '_') || sessionId;
  const ext = format === 'json' ? 'json' : format === 'csv' ? 'csv' : 'md';
  return `${baseTitle}_${timestamp}.${ext}`;
}

/**
 * 导出为 JSON 格式
 */
function exportAsJSON(data: ExportData, options: ExportOptions): ExportResult {
  const exportData = {
    session: {
      id: data.sessionId,
      title: data.title,
      projectPath: data.projectPath,
      createdAt: data.createdAt,
      updatedAt: data.updatedAt,
      ...(options.includeMetadata && { stats: data.stats })
    },
    messages: data.messages.map((msg) => {
      const item: Record<string, unknown> = {
        role: msg.role,
        content: msg.content
      };

      if (options.includeTimestamps && msg.timestamp) {
        item.timestamp = msg.timestamp;
      }

      if (options.includeCodeBlocks && msg.codeBlocks) {
        item.codeBlocks = msg.codeBlocks;
      }

      if (options.includeMetadata && msg.metadata) {
        item.metadata = msg.metadata;
      }

      return item;
    })
  };

  const content = JSON.stringify(exportData, null, 2);
  const blob = new Blob([content], { type: 'application/json' });
  const url = URL.createObjectURL(blob);

  return {
    filename: generateFilename(data.sessionId, 'json', data.title),
    content: url,
    size: blob.size,
    mimeType: 'application/json'
  };
}

/**
 * 导出为 CSV 格式
 */
function exportAsCSV(data: ExportData, options: ExportOptions): ExportResult {
  const delimiter = options.csvDelimiter || ',';
  const escapeCsv = (text: string): string => {
    if (text.includes(delimiter) || text.includes('"') || text.includes('\n')) {
      return `"${text.replace(/"/g, '""')}"`;
    }
    return text;
  };

  // CSV 头部
  const headers = ['Timestamp', 'Role', 'Content'];
  if (options.includeCodeBlocks) {
    headers.push('Code Blocks');
  }
  if (options.includeMetadata) {
    headers.push('Metadata');
  }

  let csv = headers.join(delimiter) + '\n';

  // CSV 数据行
  for (const msg of data.messages) {
    const row: string[] = [];

    // 时间戳
    if (options.includeTimestamps) {
      row.push(escapeCsv(msg.timestamp || ''));
    } else {
      row.push('');
    }

    // 角色
    row.push(escapeCsv(msg.role));

    // 内容
    const content = msg.content.replace(/\n/g, ' ').slice(0, 1000); // 限制长度并移除换行
    row.push(escapeCsv(content));

    // 代码块
    if (options.includeCodeBlocks) {
      const codeBlocks = msg.codeBlocks
        ?.map((block) => `[${block.language}] ${block.code.slice(0, 100)}...`)
        .join('; ') || '';
      row.push(escapeCsv(codeBlocks));
    }

    // 元数据
    if (options.includeMetadata) {
      const metadata = msg.metadata
        ? escapeCsv(JSON.stringify(msg.metadata))
        : '';
      row.push(metadata);
    }

    csv += row.join(delimiter) + '\n';
  }

  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);

  return {
    filename: generateFilename(data.sessionId, 'csv', data.title),
    content: url,
    size: blob.size,
    mimeType: 'text/csv'
  };
}

/**
 * 导出为 Markdown 格式
 */
function exportAsMarkdown(data: ExportData, options: ExportOptions): ExportResult {
  const headingLevel = options.markdownHeadingLevel || 1;
  const headingPrefix = '#'.repeat(headingLevel);
  const subHeadingPrefix = '#'.repeat(headingLevel + 1);
  const codePrefix = '#'.repeat(headingLevel + 2);

  let md = '';

  // 标题和元信息
  md += `${headingPrefix} ${data.title || 'Session Export'}\n\n`;

  if (options.includeMetadata) {
    md += `${subHeadingPrefix} Session Information\n\n`;
    md += `- **Session ID**: ${data.sessionId}\n`;
    if (data.projectPath) {
      md += `- **Project Path**: ${data.projectPath}\n`;
    }
    if (data.createdAt) {
      md += `- **Created**: ${data.createdAt}\n`;
    }
    if (data.updatedAt) {
      md += `- **Updated**: ${data.updatedAt}\n`;
    }
    if (data.stats) {
      md += `- **Messages**: ${data.stats.totalMessages}\n`;
      if (data.stats.totalTokens) {
        md += `- **Tokens**: ${data.stats.totalTokens}\n`;
      }
      if (data.stats.codeChanges) {
        md += `- **Code Changes**: ${data.stats.codeChanges}\n`;
      }
    }
    md += '\n';
  }

  // 消息列表
  md += `${subHeadingPrefix} Conversation\n\n`;

  for (const msg of data.messages) {
    // 消息头部
    const roleLabel =
      msg.role === 'user' ? 'User' : msg.role === 'assistant' ? 'Assistant' : 'System';
    const timestampStr =
      options.includeTimestamps && msg.timestamp
        ? ` *(${msg.timestamp})*`
        : '';

    md += `${codePrefix} ${roleLabel}${timestampStr}\n\n`;

    // 消息内容
    md += `${msg.content}\n\n`;

    // 代码块
    if (options.includeCodeBlocks && msg.codeBlocks && msg.codeBlocks.length > 0) {
      for (const block of msg.codeBlocks) {
        md += '```' + block.language + '\n';
        md += block.code;
        md += '\n```\n\n';
      }
    }

    // 元数据
    if (options.includeMetadata && msg.metadata) {
      md += `*Metadata: ${JSON.stringify(msg.metadata)}*\n\n`;
    }

    md += '---\n\n';
  }

  const blob = new Blob([md], { type: 'text/markdown;charset=utf-8;' });
  const url = URL.createObjectURL(blob);

  return {
    filename: generateFilename(data.sessionId, 'markdown', data.title),
    content: url,
    size: blob.size,
    mimeType: 'text/markdown'
  };
}

/**
 * 导出数据
 */
export function exportData(
  data: ExportData,
  options: ExportOptions
): ExportResult {
  switch (options.format) {
    case 'json':
      return exportAsJSON(data, options);
    case 'csv':
      return exportAsCSV(data, options);
    case 'markdown':
      return exportAsMarkdown(data, options);
    default:
      throw new Error(`Unsupported format: ${options.format}`);
  }
}

/**
 * 触发下载
 */
export function triggerDownload(result: ExportResult): void {
  const link = document.createElement('a');
  link.href = result.content;
  link.download = result.filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // 释放 Blob URL
  setTimeout(() => {
    URL.revokeObjectURL(result.content);
  }, 100);
}

/**
 * 批量导出
 */
export function exportBatch(
  dataList: ExportData[],
  options: ExportOptions
): ExportResult[] {
  return dataList.map((data) => exportData(data, options));
}

/**
 * 格式化文件大小
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

/**
 * 获取格式描述
 */
export function getFormatDescription(format: ExportFormat): string {
  const descriptions: Record<ExportFormat, string> = {
    json: '完整的 JSON 格式，包含所有数据和元数据',
    csv: '表格格式，适合在 Excel 等工具中查看',
    markdown: '可读的 Markdown 文档格式'
  };
  return descriptions[format];
}

/**
 * 获取格式图标
 */
export function getFormatIcon(format: ExportFormat): string {
  const icons: Record<ExportFormat, string> = {
    json: '{ }',
    csv: '📊',
    markdown: '📝'
  };
  return icons[format];
}
