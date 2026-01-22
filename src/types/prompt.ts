/**
 * 提示词优化相关类型定义
 *
 * 与 Rust 后端 optimizer::prompt_generator 和 database 模块对应
 */

/**
 * Token 统计信息
 */
export interface TokenStats {
  /**
   * 原始 Token 数
   */
  originalTokens: number;
  /**
   * 压缩后 Token 数
   */
  compressedTokens: number;
  /**
   * 节省百分比 (0.0 - 100.0)
   */
  savingsPercentage: number;
}

/**
 * 引用的会话信息
 * 与 Rust 后端 optimizer::prompt_generator::ReferencedSession 对应
 */
export interface ReferencedSession {
  /**
   * 会话 ID
   */
  sessionId: string;
  /**
   * 项目名称
   */
  projectName: string;
  /**
   * 会话摘要（可能为空）
   */
  summary: string;
  /**
   * 相似度分数 (0.0 - 1.0)
   */
  similarityScore: number;
}

/**
 * 增强提示词响应
 */
export interface EnhancedPrompt {
  /**
   * 原始目标
   */
  originalGoal: string;
  /**
   * 引用的会话
   */
  referencedSessions: ReferencedSession[];
  /**
   * 增强的提示词
   */
  enhancedPrompt: string;
  /**
   * Token 统计
   */
  tokenStats: TokenStats;
  /**
   * 置信度 (0.0 - 1.0)
   */
  confidence: number;
}

/**
 * 增强提示词请求
 */
export interface EnhancedPromptRequest {
  /**
   * 用户目标
   */
  goal: string;
  /**
   * 可选：当前跟踪的会话文件路径（首页展示的会话）
   */
  currentSessionFilePath?: string;
}

/**
 * 提示词分类
 */
export type PromptCategory = 'next_goals' | 'ai_analysis' | 'meta_template';

/**
 * 保存的提示词信息
 * 与后端 database::models::SavedPrompt 对应
 */
export interface SavedPrompt {
  /**
   * 主键 ID
   */
  id?: number;
  /**
   * 关联的会话 ID (可为空，表示全局提示词)
   */
  session_id?: string;
  /**
   * 分类
   */
  category: PromptCategory;
  /**
   * 提示词标题
   */
  title: string;
  /**
   * 提示词内容
   */
  content: string;
  /**
   * 用户评分 (1-5)
   */
  rating?: number;
  /**
   * 使用次数
   */
  usage_count: number;
  /**
   * Token 数量
   */
  tokens?: number;
  /**
   * 创建时间
   */
  created_at: string;
}

/**
 * Meta-Prompt 模板
 * 与后端 database::models::MetaTemplate 对应
 */
export interface MetaTemplate {
  /**
   * 主键 ID
   */
  id?: number;
  /**
   * 模板唯一标识
   */
  key: string;
  /**
   * 模板名称
   */
  name: string;
  /**
   * 模板内容 (支持变量占位符)
   */
  content: string;
  /**
   * 模板描述
   */
  description?: string;
  /**
   * 是否启用
   */
  is_active: boolean;
  /**
   * 创建时间
   */
  created_at?: string;
  /**
   * 更新时间
   */
  updated_at?: string;
}

/**
 * 提示词库筛选条件
 */
export interface PromptLibraryFilters {
  /**
   * 搜索关键词
   */
  search?: string;
  /**
   * 分类筛选
   */
  category?: PromptCategory | 'all';
  /**
   * 最小评分
   */
  minRating?: number;
  /**
   * 排序方式
   */
  sortBy?: 'created_at' | 'usage_count' | 'rating' | 'title';
  /**
   * 排序方向
   */
  sortOrder?: 'asc' | 'desc';
}

/**
 * 提示词库项（用于列表展示）
 */
export type PromptLibraryItem = SavedPrompt | MetaTemplate;

/**
 * 判断是否为 MetaTemplate
 */
export function isMetaTemplate(item: PromptLibraryItem): item is MetaTemplate {
  return 'key' in item && 'name' in item;
}

/**
 * 判断是否为 SavedPrompt
 */
export function isSavedPrompt(item: PromptLibraryItem): item is SavedPrompt {
  return 'category' in item && 'title' in item;
}
