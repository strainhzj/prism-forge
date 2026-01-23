/**
 * 提示词库相关类型定义
 *
 * 这些是前端特定的类型，不需要通过 ts-rs 生成
 */

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
