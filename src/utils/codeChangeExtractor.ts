/**
 * 代码变更提取器
 *
 * 从消息节点的工具调用中提取代码变更信息
 * 支持 Claude Code 的 Write/Edit 工具调用格式
 */

import type { MessageNode, CodeChangeInfo } from '@/types/message';

/**
 * Write 工具调用输入格式
 */
interface WriteToolInput {
  file_path: string;
  content: string;
}

/**
 * Edit 工具调用输入格式
 */
interface EditToolInput {
  file_path: string;
  old_text: string;
  new_text: string;
  start_line?: number;
  end_line?: number;
}

/**
 * 工具调用内容格式
 */
interface ToolCallContent {
  name: string;
  input: WriteToolInput | EditToolInput | Record<string, unknown>;
}

/**
 * 从消息节点提取代码变更
 *
 * @param node - 消息节点
 * @returns 代码变更信息数组
 */
export function extractCodeChanges(node: MessageNode): CodeChangeInfo[] {
  const changes: CodeChangeInfo[] = [];

  // 如果节点已有 metadata，直接返回
  if (node.metadata?.code_changes) {
    return node.metadata.code_changes;
  }

  // 从内容中提取工具调用
  const toolCalls = parseToolCalls(node.content || '');

  for (const toolCall of toolCalls) {
    const change = extractChangeFromToolCall(toolCall);
    if (change) {
      changes.push(change);
    }
  }

  return changes;
}

/**
 * 解析工具调用内容
 *
 * @param content - 消息内容
 * @returns 工具调用数组
 */
function parseToolCalls(content: string): ToolCallContent[] {
  const toolCalls: ToolCallContent[] = [];

  // 匹配 JSON 格式的工具调用
  // 格式: {"name": "tool_name", "input": {...}}
  const jsonPattern = /\{\s*"name"\s*:\s*"([^"]+)"\s*,\s*"input"\s*:\s*(\{[^}]*\})\s*\}/g;
  let match;

  while ((match = jsonPattern.exec(content)) !== null) {
    const name = match[1];
    try {
      const input = JSON.parse(match[2]);
      toolCalls.push({ name, input });
    } catch {
      // JSON 解析失败，跳过
      continue;
    }
  }

  return toolCalls;
}

/**
 * 从工具调用中提取代码变更
 *
 * @param toolCall - 工具调用
 * @returns 代码变更信息或 null
 */
function extractChangeFromToolCall(toolCall: ToolCallContent): CodeChangeInfo | null {
  const { name, input } = toolCall;

  // 处理 Write 工具
  if (name === 'write' || name === 'create') {
    return extractWriteChange(input as Record<string, unknown>);
  }

  // 处理 Edit 工具
  if (name === 'edit' || name === 'update') {
    return extractEditChange(input as Record<string, unknown>);
  }

  return null;
}

/**
 * 从 Write 工具提取变更
 *
 * @param input - 工具输入
 * @returns 代码变更信息
 */
function extractWriteChange(input: Record<string, unknown>): CodeChangeInfo | null {
  const filePath = input.file_path as string;
  const content = input.content as string;

  if (!filePath) {
    return null;
  }

  return {
    file_path: filePath,
    change_type: 'create',
    lines_added: content ? content.split('\n').length : 0,
    new_text: content,
    tool_name: 'write',
  };
}

/**
 * 从 Edit 工具提取变更
 *
 * @param input - 工具输入
 * @returns 代码变更信息
 */
function extractEditChange(input: Record<string, unknown>): CodeChangeInfo | null {
  const filePath = input.file_path as string;
  const oldText = input.old_text as string;
  const newText = input.new_text as string;
  const startLine = input.start_line as number | undefined;
  const endLine = input.end_line as number | undefined;

  if (!filePath || (!oldText && !newText)) {
    return null;
  }

  return {
    file_path: filePath,
    change_type: 'update',
    lines_added: newText ? newText.split('\n').length : 0,
    lines_removed: oldText ? oldText.split('\n').length : 0,
    old_text: oldText,
    new_text: newText,
    start_line: startLine,
    end_line: endLine,
    tool_name: 'edit',
  };
}

/**
 * 检查消息节点是否包含代码变更
 *
 * @param node - 消息节点
 * @returns 是否包含代码变更
 */
export function hasCodeChanges(node: MessageNode): boolean {
  const changes = extractCodeChanges(node);
  return changes.length > 0;
}

/**
 * 获取消息节点中的所有变更文件路径
 *
 * @param node - 消息节点
 * @returns 文件路径数组
 */
export function getChangedFilePaths(node: MessageNode): string[] {
  const changes = extractCodeChanges(node);
  return changes.map((change) => change.file_path);
}

/**
 * 按文件路径分组代码变更
 *
 * @param node - 消息节点
 * @returns 文件路径到变更信息的映射
 */
export function groupChangesByFile(node: MessageNode): Map<string, CodeChangeInfo[]> {
  const changes = extractCodeChanges(node);
  const grouped = new Map<string, CodeChangeInfo[]>();

  for (const change of changes) {
    const existing = grouped.get(change.file_path) || [];
    existing.push(change);
    grouped.set(change.file_path, existing);
  }

  return grouped;
}

/**
 * 从消息树中提取所有代码变更
 *
 * @param node - 消息节点（根节点）
 * @returns 所有代码变更信息数组
 */
export function extractAllCodeChanges(node: MessageNode): CodeChangeInfo[] {
  const changes: CodeChangeInfo[] = [];

  // 提取当前节点的变更
  changes.push(...extractCodeChanges(node));

  // 递归提取子节点的变更
  if (node.children && node.children.length > 0) {
    for (const child of node.children) {
      changes.push(...extractAllCodeChanges(child));
    }
  }

  return changes;
}

/**
 * 计算代码变更统计
 *
 * @param node - 消息节点
 * @returns 变更统计信息
 */
export interface ChangeStatistics {
  total_files: number;
  files_created: number;
  files_updated: number;
  files_deleted: number;
  lines_added: number;
  lines_removed: number;
}

export function calculateChangeStatistics(node: MessageNode): ChangeStatistics {
  const changes = extractAllCodeChanges(node);
  const stats: ChangeStatistics = {
    total_files: 0,
    files_created: 0,
    files_updated: 0,
    files_deleted: 0,
    lines_added: 0,
    lines_removed: 0,
  };

  const uniqueFiles = new Set<string>();

  for (const change of changes) {
    uniqueFiles.add(change.file_path);
    stats.lines_added += change.lines_added || 0;
    stats.lines_removed += change.lines_removed || 0;

    if (change.change_type === 'create') {
      stats.files_created++;
    } else if (change.change_type === 'update') {
      stats.files_updated++;
    } else if (change.change_type === 'delete') {
      stats.files_deleted++;
    }
  }

  stats.total_files = uniqueFiles.size;

  return stats;
}

/**
 * 更新消息节点的 metadata，添加代码变更信息
 *
 * @param node - 消息节点
 * @returns 更新后的消息节点（不修改原对象）
 */
export function enrichWithCodeChanges(node: MessageNode): MessageNode {
  const changes = extractCodeChanges(node);

  return {
    ...node,
    metadata: {
      ...node.metadata,
      code_changes: changes,
    },
  };
}
