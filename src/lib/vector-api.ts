/**
 * 向量搜索 API - Tauri invoke 封装
 *
 * 提供与 Rust 后端向量搜索功能的接口
 */

import { invoke } from '@tauri-apps/api/core';

// ==================== 类型定义 ====================

/**
 * 语义搜索请求参数
 */
export interface SemanticSearchRequest {
  /** 搜索查询文本 */
  query: string;
  /** 返回结果数量（默认 10） */
  topK?: number;
  /** 最小相似度阈值（0.0-1.0，默认 0.0） */
  minSimilarity?: number;
}

/**
 * 会话信息
 */
export interface SessionInfo {
  /** 会话 ID */
  sessionId: string;
  /** 项目路径 */
  projectPath: string;
  /** 项目名称 */
  projectName: string;
  /** 文件路径 */
  filePath: string;
  /** 用户评分（1-5） */
  rating: number | null;
  /** 标签数组 */
  tags: string[];
}

/**
 * 语义搜索结果
 */
export interface SemanticSearchResult {
  /** 会话信息 */
  session: SessionInfo;
  /** 相似度分数（0.0-1.0） */
  similarityScore: number;
  /** 会话摘要 */
  summary: string;
}

/**
 * 向量设置
 */
export interface VectorSettings {
  /** 是否启用向量搜索 */
  vectorSearchEnabled: boolean;
  /** Embedding 提供商（openai/fastembed） */
  embeddingProvider: string;
  /** Embedding 模型名称 */
  embeddingModel: string;
  /** 批量处理大小 */
  embeddingBatchSize: number;
}

// ==================== API 函数 ====================

/**
 * 语义搜索
 *
 * @param request - 搜索请求参数
 * @returns 搜索结果列表
 */
export async function semanticSearch(
  request: SemanticSearchRequest
): Promise<SemanticSearchResult[]> {
  return await invoke<SemanticSearchResult[]>('semantic_search', {
    request: {
      query: request.query,
      topK: request.topK,
      minSimilarity: request.minSimilarity,
    },
  });
}

/**
 * 查找相似会话
 *
 * @param sessionId - 目标会话 ID
 * @param topK - 返回结果数量（默认 10）
 * @param minSimilarity - 最小相似度阈值（默认 0.0）
 * @returns 相似会话列表
 */
export async function findSimilarSessions(
  sessionId: string,
  topK?: number,
  minSimilarity?: number
): Promise<SemanticSearchResult[]> {
  return await invoke<SemanticSearchResult[]>('find_similar_sessions', {
    sessionId,
    topK,
    minSimilarity,
  });
}

/**
 * 获取向量设置
 *
 * @returns 向量设置
 */
export async function getVectorSettings(): Promise<VectorSettings> {
  return await invoke<VectorSettings>('get_vector_settings');
}

/**
 * 更新向量设置
 *
 * @param settings - 向量设置
 */
export async function updateVectorSettings(
  settings: VectorSettings
): Promise<void> {
  return await invoke('update_vector_settings', { settings });
}

/**
 * 手动触发向量同步
 *
 * @returns 成功向量化的会话数量
 */
export async function syncEmbeddingsNow(): Promise<number> {
  return await invoke<number>('sync_embeddings_now');
}
