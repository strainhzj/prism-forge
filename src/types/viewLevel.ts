/**
 * 视图等级类型定义
 *
 * 与 Rust 后端 parser::view_level::ViewLevel 对应
 */

/**
 * 视图等级枚举
 */
export enum ViewLevel {
  /** 完整模式 - 显示所有消息类型 */
  Full = 'full',
  /** 会话模式 - 仅显示用户、助手和思考消息 */
  Conversation = 'conversation',
  /** 问答对模式 - 提取用户问题和最终答案 */
  QAPairs = 'qa_pairs',
  /** 仅助手模式 - 仅显示助手输出 */
  AssistantOnly = 'assistant_only',
  /** 仅用户模式 - 仅显示用户输入 */
  UserOnly = 'user_only',
}

/**
 * 视图等级显示信息
 */
export interface ViewLevelInfo {
  /** 视图等级值 */
  value: ViewLevel;
  /** 显示名称（用户可见） */
  displayName: string;
  /** 描述信息 */
  description: string;
  /** 图标（可选） */
  icon?: string;
}

/**
 * 问答对
 */
export interface QAPair {
  /** 用户问题 */
  question: Message;
  /** 助手最终回复（可能为 null） */
  answer: Message | null;
  /** 问答对的时间戳 */
  timestamp: string;
}

/**
 * 消息（简化版）
 */
export interface Message {
  /** 数据库 ID */
  id?: number | null;
  /** 会话 ID */
  session_id: string;
  /** 消息 UUID */
  uuid: string;
  /** 父消息 UUID */
  parent_uuid: string | null;
  /** 消息类型（user/assistant/thinking等） */
  msg_type: string;
  /** 时间戳 */
  timestamp: string;
  /** 文件偏移量 */
  offset: number;
  /** 消息长度 */
  length: number;
  /** 摘要/内容 */
  summary?: string | null;
  /** 父索引 */
  parent_idx?: number | null;
  /** 创建时间 */
  created_at: string;
}

/**
 * 导出格式类型
 */
export enum ExportFormatType {
  /** Markdown 格式 */
  Markdown = 'markdown',
  /** JSON 格式 */
  Json = 'json',
}

/**
 * 视图等级偏好设置
 */
export interface ViewLevelPreference {
  /** 会话 ID */
  session_id: string;
  /** 视图等级 */
  view_level: ViewLevel;
  /** 创建时间 */
  created_at: string;
  /** 更新时间 */
  updated_at: string;
}

/**
 * 视图等级映射表（用于显示）
 */
export const VIEW_LEVEL_INFO: Record<ViewLevel, ViewLevelInfo> = {
  [ViewLevel.Full]: {
    value: ViewLevel.Full,
    displayName: '完整模式',
    description: '显示所有消息类型，包括工具调用、错误等',
    icon: '📄',
  },
  [ViewLevel.Conversation]: {
    value: ViewLevel.Conversation,
    displayName: '会话模式',
    description: '仅显示用户、助手和思考消息',
    icon: '💬',
  },
  [ViewLevel.QAPairs]: {
    value: ViewLevel.QAPairs,
    displayName: '问答对模式',
    description: '提取用户问题和最终答案，忽略中间思考过程',
    icon: '❓',
  },
  [ViewLevel.AssistantOnly]: {
    value: ViewLevel.AssistantOnly,
    displayName: '仅助手',
    description: '仅显示助手的回复',
    icon: '🤖',
  },
  [ViewLevel.UserOnly]: {
    value: ViewLevel.UserOnly,
    displayName: '仅用户',
    description: '仅显示用户的输入',
    icon: '👤',
  },
};

/**
 * 获取视图等级的显示信息
 */
export function getViewLevelInfo(viewLevel: ViewLevel): ViewLevelInfo {
  return VIEW_LEVEL_INFO[viewLevel];
}

/**
 * 获取所有视图等级选项
 */
export function getViewLevelOptions(): ViewLevelInfo[] {
  return Object.values(VIEW_LEVEL_INFO);
}
