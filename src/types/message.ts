/**
 * 消息树类型定义
 *
 * 与 Rust 后端 ConversationTree 和 MessageNode 对应
 */

/**
 * 消息元数据
 */
export interface MessageMetadata {
  /**
   * 工具调用信息
   */
  tool_calls?: ToolCallInfo[];
  /**
   * 错误信息
   */
  errors?: ErrorInfo[];
  /**
   * 代码变更信息
   */
  code_changes?: CodeChangeInfo[];
}

/**
 * 工具调用信息
 */
export interface ToolCallInfo {
  name: string;
  input?: string;
  output?: string;
  duration?: number;
}

/**
 * 错误信息
 */
export interface ErrorInfo {
  message: string;
  stack_trace?: string;
}


/**
 * 代码变更信息
 */
export interface CodeChangeInfo {
  file_path: string;
  change_type: 'create' | 'update' | 'delete';
  lines_added?: number;
  lines_removed?: number;
  /**
   * 变更前的代码（用于 Diff 视图）
   */
  old_text?: string;
  /**
   * 变更后的代码（用于 Diff 视图）
   */
  new_text?: string;
  /**
   * 起始行号（从 1 开始）
   */
  start_line?: number;
  /**
   * 结束行号
   */
  end_line?: number;
  /**
   * 工具名称（如 'write', 'edit'）
   */
  tool_name?: string;
}
/**
 * 消息节点
 */
export interface MessageNode {
  /**
   * 消息唯一标识（UUID）
   */
  id: string;
  /**
   * 父消息 ID（None 表示根节点）
   */
  parent_id: string | null;
  /**
   * 树深度（0 表示根节点）
   */
  depth: number;
  /**
   * 消息角色（User, Assistant, System, Tool）
   */
  role?: string;
  /**
   * 消息类型（message, tool_result, text, etc.）
   */
  type?: string;
  /**
   * 消息内容（截断后的显示内容）
   */
  content?: string;
  /**
   * 完整内容（用于详情查看）
   */
  fullContent?: string;
  /**
   * 消息类型（缓存的 type 字段）
   */
  msgType?: string;
  /**
   * 时间戳
   */
  timestamp?: string;
  /**
   * 子消息列表
   */
  children: MessageNode[];
  /**
   * 线程 ID（支持多线程对话）
   */
  thread_id: string | null;
  /**
   * 提取的元数据
   */
  metadata?: MessageMetadata;
  /**
   * 原始 JSON 数据（用于存储未解析的字段）
   */
  [key: string]: any;
}

/**
 * 对话树
 */
export interface ConversationTree {
  /**
   * 根节点列表（User 消息）
   */
  roots: MessageNode[];
  /**
   * 总消息数量
   */
  total_count: number;
  /**
   * 最大深度
   */
  max_depth: number;
  /**
   * 线程数量（支持多线程对话）
   */
  thread_count: number;
}

/**
 * 消息节点展开状态
 */
export interface NodeExpansionState {
  [nodeId: string]: boolean;
}

/**
 * 消息节点内容缓存（用于懒加载）
 */
export interface NodeContentCache {
  [nodeId: string]: MessageNode;
}
