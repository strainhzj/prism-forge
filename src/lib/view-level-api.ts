/**
 * 多级日志读取 API - Tauri invoke 封装
 *
 * 提供与 Rust 后端多级日志读取功能的接口
 */

import { invoke } from '@tauri-apps/api/core';
import { ViewLevel, type Message, type QAPair } from '@/types/viewLevel';

// ==================== 类型定义 ====================

/**
 * 导出格式类型
 */
export type ExportFormat = 'markdown' | 'json';

// ==================== API 函数 ====================

/**
 * 根据视图等级获取消息列表
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 * @returns 过滤后的消息列表
 */
export async function getMessagesByLevel(
  sessionId: string,
  viewLevel: ViewLevel
): Promise<Message[]> {
  try {
    const messages = await invoke<Message[]>('cmd_get_messages_by_level', {
      sessionId,
      viewLevel,
    });
    return messages;
  } catch (error) {
    console.error('获取消息失败:', error);
    throw new Error(`获取消息失败: ${error}`);
  }
}

/**
 * 根据视图等级提取问答对
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级（必须是 QAPairs）
 * @returns 问答对列表
 */
export async function getQAPairsByLevel(
  sessionId: string,
  viewLevel: ViewLevel
): Promise<QAPair[]> {
  try {
    const qaPairs = await invoke<QAPair[]>('cmd_get_qa_pairs_by_level', {
      sessionId,
      viewLevel,
    });
    return qaPairs;
  } catch (error) {
    console.error('获取问答对失败:', error);
    throw new Error(`获取问答对失败: ${error}`);
  }
}

/**
 * 保存视图等级偏好设置
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 */
export async function saveViewLevelPreference(
  sessionId: string,
  viewLevel: ViewLevel
): Promise<void> {
  try {
    await invoke('cmd_save_view_level_preference', {
      sessionId,
      viewLevel,
    });
  } catch (error) {
    console.error('保存视图等级偏好失败:', error);
    throw new Error(`保存视图等级偏好失败: ${error}`);
  }
}

/**
 * 获取视图等级偏好设置
 *
 * @param sessionId - 会话 ID
 * @returns 视图等级（如果不存在则返回默认值 Full）
 */
export async function getViewLevelPreference(
  sessionId: string
): Promise<ViewLevel> {
  try {
    const viewLevel = await invoke<ViewLevel>('cmd_get_view_level_preference', {
      sessionId,
    });
    return viewLevel;
  } catch (error) {
    console.error('获取视图等级偏好失败:', error);
    // 如果获取失败，返回默认值
    return ViewLevel.Full;
  }
}

/**
 * 导出会话（按视图等级过滤）
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 * @param format - 导出格式（markdown 或 json）
 * @returns 导出的内容字符串
 */
export async function exportSessionByLevel(
  sessionId: string,
  viewLevel: ViewLevel,
  format: ExportFormat
): Promise<string> {
  try {
    const content = await invoke<string>('cmd_export_session_by_level', {
      sessionId,
      viewLevel,
      format,
    });
    return content;
  } catch (error) {
    console.error('导出会话失败:', error);
    throw new Error(`导出会话失败: ${error}`);
  }
}

// ==================== 组合 API 函数 ====================

/**
 * 自动加载会话的视图等级偏好，如果不存在则使用默认值
 *
 * @param sessionId - 会话 ID
 * @param defaultLevel - 默认视图等级（默认 Full）
 * @returns 视图等级
 */
export async function loadViewLevelWithDefault(
  sessionId: string,
  defaultLevel: ViewLevel = ViewLevel.Full
): Promise<ViewLevel> {
  try {
    return await getViewLevelPreference(sessionId);
  } catch (error) {
    console.warn('加载视图等级偏好失败，使用默认值:', error);
    return defaultLevel;
  }
}

/**
 * 保存并应用视图等级偏好
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 * @returns 是否保存成功
 */
export async function applyViewLevel(
  sessionId: string,
  viewLevel: ViewLevel
): Promise<boolean> {
  try {
    await saveViewLevelPreference(sessionId, viewLevel);
    return true;
  } catch (error) {
    console.error('应用视图等级失败:', error);
    return false;
  }
}

/**
 * 批量导出会话（支持多种格式）
 *
 * @param sessionId - 会话 ID
 * @param viewLevel - 视图等级
 * @param formats - 导出格式数组
 * @returns 格式到内容的映射
 */
export async function batchExportSession(
  sessionId: string,
  viewLevel: ViewLevel,
  formats: ExportFormat[]
): Promise<Map<ExportFormat, string>> {
  const results = new Map<ExportFormat, string>();

  for (const format of formats) {
    try {
      const content = await exportSessionByLevel(sessionId, viewLevel, format);
      results.set(format, content);
    } catch (error) {
      console.error(`导出 ${format} 格式失败:`, error);
      // 继续导出其他格式
    }
  }

  return results;
}

// ==================== 错误处理工具 ====================

/**
 * 判断错误是否为"会话不存在"错误
 */
export function isSessionNotFoundError(error: unknown): boolean {
  if (error instanceof Error) {
    return error.message.includes('会话不存在') ||
           error.message.includes('Session not found');
  }
  return false;
}

/**
 * 判断错误是否为"文件不存在"错误
 */
export function isFileNotFoundError(error: unknown): boolean {
  if (error instanceof Error) {
    return error.message.includes('文件不存在') ||
           error.message.includes('file not found');
  }
  return false;
}
