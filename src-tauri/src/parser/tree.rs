//! 消息树构建模块
//!
//! 负责将 JSONL 解析的消息列表重构为嵌套的树结构。
//! 使用迭代算法构建消息树，避免深层递归导致的栈溢出。

use std::collections::{HashMap, HashSet};
use anyhow::{Result, Context};
use serde_json::Value;
use serde::{Serialize, Deserialize};

use super::jsonl::JsonlEntry;
use super::extractor::MetadataExtractor;

/// 消息元数据
///
/// 从消息内容中提取的关键信息，用于快速检索和展示
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageMetadata {
    /// 消息摘要（用于列表展示）
    pub summary: Option<String>,

    /// 工具调用列表
    pub tool_calls: Vec<ToolCall>,

    /// 错误消息列表
    pub errors: Vec<ErrorMessage>,

    /// 代码变更列表
    pub code_changes: Vec<CodeChange>,
}

/// 工具调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// 工具名称（如 "read_file", "write_file"）
    pub name: String,

    /// 工具输入参数
    pub input: serde_json::Value,

    /// 调用状态（success/error）
    pub status: String,
}

/// 错误消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    /// 错误类型
    pub error_type: String,

    /// 错误消息内容
    pub message: String,

    /// 相关的工具名称（如果有）
    pub related_tool: Option<String>,
}

/// 代码变更记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeChange {
    /// 操作类型（Read/Write/Edit）
    pub operation: String,

    /// 文件路径
    pub file_path: String,

    /// 变更的行数（估算）
    pub lines_changed: Option<usize>,
}

/// 消息树节点
///
/// 表示 Claude Code 会话中的一条消息，包含其内容和子消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageNode {
    /// 消息唯一标识（UUID）
    pub id: String,

    /// 父消息 ID（None 表示根节点）
    pub parent_id: Option<String>,

    /// 树深度（0 表示根节点）
    pub depth: usize,

    /// 原始 JSONL 条目数据
    #[serde(flatten)]
    pub message_data: Value,

    /// 子消息列表
    pub children: Vec<MessageNode>,

    /// 线程 ID（支持多线程对话，如并行的工具调用）
    pub thread_id: Option<String>,

    /// 提取的元数据（工具调用、错误、代码变更等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MessageMetadata>,
}

impl MessageNode {
    /// 创建新的消息节点
    pub fn new(
        id: String,
        parent_id: Option<String>,
        message_data: Value,
    ) -> Self {
        Self {
            id,
            parent_id,
            depth: 0,
            message_data,
            children: Vec::new(),
            thread_id: None,
            metadata: None,
        }
    }

    /// 添加子节点
    pub fn add_child(&mut self, child: MessageNode) {
        self.children.push(child);
    }

    /// 获取消息类型
    pub fn message_type(&self) -> Option<String> {
        self.message_data.get("type")?.as_str().map(|s| s.to_string())
    }

    /// 获取消息角色
    pub fn role(&self) -> Option<String> {
        self.message_data.get("role")?.as_str().map(|s| s.to_string())
    }

    /// 检查是否为工具调用消息
    pub fn is_tool_use(&self) -> bool {
        self.message_type().as_deref() == Some("tool_use")
    }

    /// 检查是否为用户消息
    pub fn is_user_message(&self) -> bool {
        self.role().as_deref() == Some("user")
    }

    /// 检查是否为助手消息
    pub fn is_assistant_message(&self) -> bool {
        self.role().as_deref() == Some("assistant")
    }
}

/// 完整的对话树
///
/// 包含根节点列表和相关的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationTree {
    /// 根节点列表（User 消息）
    pub roots: Vec<MessageNode>,

    /// 总消息数量
    pub total_count: usize,

    /// 最大深度
    pub max_depth: usize,

    /// 线程数量（支持多线程对话）
    pub thread_count: usize,
}

impl ConversationTree {
    /// 创建新的对话树
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            total_count: 0,
            max_depth: 0,
            thread_count: 0,
        }
    }

    /// 添加根节点
    pub fn add_root(&mut self, root: MessageNode) {
        self.total_count += 1 + self.count_descendants(&root);
        self.max_depth = self.max_depth.max(self.calculate_depth(&root, 0));
        self.roots.push(root);
    }

    /// 计算节点深度
    fn calculate_depth(&self, node: &MessageNode, current_depth: usize) -> usize {
        let mut max_child_depth = current_depth;
        for child in &node.children {
            let child_depth = self.calculate_depth(child, current_depth + 1);
            max_child_depth = max_child_depth.max(child_depth);
        }
        max_child_depth
    }

    /// 统计后代节点数量
    fn count_descendants(&self, node: &MessageNode) -> usize {
        let mut count = node.children.len();
        for child in &node.children {
            count += self.count_descendants(child);
        }
        count
    }
}

/// 消息树构建器
///
/// 从 JSONL 条目列表构建消息树，使用迭代算法避免栈溢出
pub struct MessageTreeBuilder {
    /// ID 到节点的映射
    node_map: HashMap<String, MessageNode>,

    /// 子节点到父节点的映射
    child_to_parent: HashMap<String, String>,

    /// 根节点 ID 集合（没有父节点的节点）
    root_ids: HashSet<String>,
}

impl MessageTreeBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            node_map: HashMap::new(),
            child_to_parent: HashMap::new(),
            root_ids: HashSet::new(),
        }
    }

    /// 从 JSONL 条目列表构建消息树
    ///
    /// # 参数
    /// * `entries` - JSONL 条目列表
    ///
    /// # 返回
    /// 返回完整的对话树或错误
    ///
    /// # 算法说明
    /// 使用迭代算法（而非递归）构建树：
    /// 1. 第一遍扫描：创建所有节点并建立父子关系映射
    /// 2. 第二遍扫描：从根节点开始，使用栈构建完整的树结构
    ///
    /// # 示例
    /// ```rust
    /// let mut parser = JsonlParser::new(file_path)?;
    /// let entries = parser.parse_all()?;
    /// let tree = MessageTreeBuilder::build_from_entries(&entries)?;
    /// ```
    pub fn build_from_entries(entries: &[JsonlEntry]) -> Result<ConversationTree> {
        let mut builder = Self::new();

        // 第一遍扫描：创建所有节点
        for entry in entries {
            // 提取消息 ID（uuid 字段）
            let id = entry.data.get("uuid")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("消息缺少 uuid 字段"))?
                .to_string();

            // 提取父消息 ID（parentUuid 字段，可能不存在）
            let parent_id = entry.data.get("parentUuid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // 创建消息节点
            let node = MessageNode::new(
                id.clone(),
                parent_id.clone(),
                entry.data.clone(),
            );

            // 添加到节点映射
            builder.node_map.insert(id.clone(), node);

            // 记录父子关系
            if let Some(ref parent) = parent_id {
                builder.child_to_parent.insert(id.clone(), parent.clone());
            } else {
                // 没有父节点，是根节点
                builder.root_ids.insert(id.clone());
            }
        }

        // 第二遍扫描：构建树结构（迭代方式，使用栈）
        let mut tree = ConversationTree::new();

        // 处理每个根节点
        for root_id in &builder.root_ids {
            if let Some(root_node) = builder.node_map.get(root_id) {
                // 克隆根节点（因为我们需要修改它）
                let mut built_root = builder.build_tree_iterative(root_node)?;

                // 只保留 User 消息作为根节点（过滤掉其他类型的根节点）
                if built_root.is_user_message() {
                    tree.add_root(built_root);
                }
            }
        }

        // 统计线程数量
        tree.thread_count = builder.count_threads(&tree);

        // 提取元数据（工具调用、错误、代码变更、摘要）
        MetadataExtractor::extract_tree_metadata(&mut tree)
            .context("提取消息元数据失败")?;

        Ok(tree)
    }

    /// 使用迭代算法构建单棵树（避免递归栈溢出）
    ///
    /// # 参数
    /// * `root` - 根节点
    ///
    /// # 返回
    /// 返回构建完成的树节点
    fn build_tree_iterative(&self, root: &MessageNode) -> Result<MessageNode> {
        // 第一步：建立父子关系映射
        let mut child_map: HashMap<String, Vec<String>> = HashMap::new();
        for (node_id, node) in &self.node_map {
            if let Some(ref parent_id) = node.parent_id {
                child_map.entry(parent_id.clone())
                    .or_insert_with(Vec::new)
                    .push(node_id.clone());
            }
        }

        // 第二步：使用迭代方式构建树
        // 使用栈存储需要处理的节点：(节点ID, 深度)
        let mut processing_stack: Vec<(String, usize)> = vec![(root.id.clone(), 0)];
        let mut built_nodes: HashMap<String, MessageNode> = HashMap::new();

        // 按深度优先顺序处理节点
        let mut processing_order: Vec<(String, usize)> = Vec::new();

        while let Some((node_id, depth)) = processing_stack.pop() {
            processing_order.push((node_id.clone(), depth));

            // 将子节点加入栈
            if let Some(children) = child_map.get(&node_id) {
                for child_id in children {
                    processing_stack.push((child_id.clone(), depth + 1));
                }
            }
        }

        // 反向遍历（从叶子节点开始构建）
        for (node_id, depth) in processing_order.into_iter().rev() {
            if let Some(node_data) = self.node_map.get(&node_id) {
                let mut new_node = node_data.clone();
                new_node.depth = depth;

                // 添加已构建的子节点
                if let Some(children_ids) = child_map.get(&node_id) {
                    for child_id in children_ids {
                        if let Some(child) = built_nodes.remove(child_id) {
                            new_node.children.push(child);
                        }
                    }
                }

                built_nodes.insert(node_id, new_node);
            }
        }

        // 返回根节点
        built_nodes.remove(&root.id)
            .ok_or_else(|| anyhow::anyhow!("根节点未找到"))
    }

    /// 统计线程数量
    fn count_threads(&self, tree: &ConversationTree) -> usize {
        let mut threads = HashSet::new();

        fn collect_threads(node: &MessageNode, threads: &mut HashSet<String>) {
            if let Some(ref thread_id) = node.thread_id {
                threads.insert(thread_id.clone());
            }
            for child in &node.children {
                collect_threads(child, threads);
            }
        }

        for root in &tree.roots {
            collect_threads(root, &mut threads);
        }

        threads.len()
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::jsonl::JsonlEntry;
    use serde_json::json;

    /// 创建测试用的 JSONL 条目
    fn create_test_entry(uuid: &str, parent_uuid: Option<&str>, role: &str) -> JsonlEntry {
        let mut data = json!({
            "uuid": uuid,
            "role": role,
            "type": "message",
            "content": [{"type": "text", "text": "Test message"}]
        });

        if let Some(parent) = parent_uuid {
            data["parentUuid"] = json!(parent);
        }

        JsonlEntry::new(0, 0, data)
    }

    #[test]
    fn test_build_simple_tree() {
        // 创建简单的树结构：
        // User1 (根)
        //   └─ Assistant1
        //        └─ User2

        let entries = vec![
            create_test_entry("user1", None, "user"),
            create_test_entry("assistant1", Some("user1"), "assistant"),
            create_test_entry("user2", Some("assistant1"), "user"),
        ];

        let tree = MessageTreeBuilder::build_from_entries(&entries).unwrap();

        // 验证：应该有 1 个根节点
        assert_eq!(tree.roots.len(), 1);

        // 验证：根节点是用户消息
        assert!(tree.roots[0].is_user_message());
        assert_eq!(tree.roots[0].id, "user1");

        // 验证：深度正确
        assert_eq!(tree.roots[0].depth, 0);
        assert_eq!(tree.roots[0].children[0].depth, 1);
    }

    #[test]
    fn test_build_multiple_roots() {
        // 创建多个根节点：
        // User1 (根1)
        // User2 (根2)

        let entries = vec![
            create_test_entry("user1", None, "user"),
            create_test_entry("user2", None, "user"),
        ];

        let tree = MessageTreeBuilder::build_from_entries(&entries).unwrap();

        // 应该有 2 个根节点
        assert_eq!(tree.roots.len(), 2);
        assert_eq!(tree.total_count, 2);
    }

    #[test]
    fn test_deep_nesting() {
        // 创建深度嵌套的树（测试迭代算法不会栈溢出）
        let mut entries = vec![create_test_entry("root", None, "user")];

        // 创建 100 层深度
        let mut parent_id = "root".to_string();
        for i in 1..=100 {
            let id = format!("node_{}", i);
            entries.push(create_test_entry(&id, Some(&parent_id), "assistant"));
            parent_id = id;
        }

        let tree = MessageTreeBuilder::build_from_entries(&entries).unwrap();

        // 验证：深度应该大于 100
        assert!(tree.max_depth >= 100);

        // 验证：总节点数正确
        assert_eq!(tree.total_count, 101);
    }

    #[test]
    fn test_message_type_detection() {
        let entries = vec![
            create_test_entry("user1", None, "user"),
            create_test_entry("assistant1", Some("user1"), "assistant"),
        ];

        let tree = MessageTreeBuilder::build_from_entries(&entries).unwrap();

        assert!(tree.roots[0].is_user_message());
        assert!(!tree.roots[0].is_assistant_message());

        assert!(tree.roots[0].children[0].is_assistant_message());
        assert!(!tree.roots[0].children[0].is_user_message());
    }
}
