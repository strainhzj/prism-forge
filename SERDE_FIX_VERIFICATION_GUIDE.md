# 字段名序列化修复验证指南

## 📋 修复概述

**问题**：Rust 结构体只有 `#[ts(rename_all = "camelCase")]`，缺少 `#[serde(rename_all = "camelCase")]`，导致前后端字段名不一致。

**修复方案**：为所有意图分析相关的 Rust 结构体添加 `#[serde(rename_all = "camelCase")]`。

**修复文件**：
- `src-tauri/src/intent_analyzer/qa_detector.rs` (DecisionQAPair, QAPairContext)
- `src-tauri/src/intent_analyzer/opening_intent.rs` (OpeningIntent)
- `src-tauri/src/intent_analyzer/decision_analyzer.rs` (DecisionAnalysis, DecisionType, Alternative)
- `src-tauri/src/database/intent_analysis_repository.rs` (IntentAnalysisHistory)
- `src/components/intent/DecisionList.tsx` (移除 convertToSnakeCase 函数)
- `src/lib/intentMapper.ts` (修复类型匹配问题)

---

## ✅ 编译验证

### 1. Rust 编译检查

```bash
cd src-tauri
cargo check
```

**预期结果**：
- ✅ 编译成功，只有警告无错误
- ✅ 没有 serde trait bound 错误

### 2. TypeScript 类型重新生成

```bash
cd src-tauri
cargo run --bin generate_types
```

**预期结果**：
- ✅ 生成成功，无错误
- ✅ 生成的类型使用 camelCase

### 3. 验证生成的类型

检查以下文件：
- `src/types/generated/DecisionQAPair.ts`
- `src/types/generated/OpeningIntent.ts`
- `src/types/generated/IntentAnalysisHistory.ts`
- `src/types/generated/DecisionAnalysis.ts`

**预期内容**：
```typescript
// DecisionQAPair.ts
export interface DecisionQAPair {
  qaIndex: number;  // ✅ camelCase
  assistantAnswerUuid: string;  // ✅ camelCase
  // ...
}

// OpeningIntent.ts
export interface OpeningIntent {
  intentType: string;  // ✅ camelCase
  confidence: number;  // ✅ camelCase
  // ...
}
```

### 4. 前端构建验证

```bash
npm run build
```

**预期结果**：
- ✅ TypeScript 编译成功
- ✅ 无类型错误
- ✅ Vite 打包成功

---

## 🧪 功能验证步骤

### 步骤 1：启动应用

```bash
npm run tauri dev
```

### 步骤 2：加载测试会话

1. 打开应用
2. 导航到"会话意图分析"页面
3. 选择一个包含意图分析历史的会话

### 步骤 3：验证左侧会话历史

**预期结果**：
- ✅ 显示"开场白已分析"而不是"N/A"
- ✅ 显示开场白意图类型（如"bug_fix"）
- ✅ 显示置信度（如"0.85"）

### 步骤 4：验证中间问答对列表

**预期结果**：
- ✅ 问答对编号正确显示（Q&A #1, Q&A #2...）
- ✅ 不显示"#NaN"
- ✅ 点击问答对后右侧显示决策分析

### 步骤 5：验证右侧决策分析

**预期结果**：
- ✅ 点击问答对后，右侧显示"正在分析决策..."
- ✅ 分析完成后显示决策内容
- ✅ 不出现"missing field 'qa_index'"错误

### 步骤 6：检查开发者控制台

打开浏览器开发者工具（F12），检查控制台：

**预期结果**：
- ✅ 没有"missing field 'qa_index'"错误
- ✅ 没有类型不匹配警告
- ✅ 调试日志显示 camelCase 字段名

---

## 🔍 数据流验证

### 验证点 1：Rust 序列化输出

在 Rust 代码中添加调试日志：

```rust
// src-tauri/src/database/intent_analysis_repository.rs
#[cfg(debug_assertions)]
{
    eprintln!("序列化 IntentAnalysisHistory:");
    eprintln!("{}", serde_json::to_string_pretty(&history).unwrap());
}
```

**预期输出**：
```json
{
  "sessionId": "...",
  "qaPairs": [
    {
      "qaIndex": 0,  // ✅ camelCase
      "assistantAnswerUuid": "...",  // ✅ camelCase
      // ...
    }
  ],
  "openingIntent": {
    "intentType": "...",  // ✅ camelCase
    "confidence": 0.85  // ✅ camelCase
  }
}
```

### 验证点 2：前端接收数据

在浏览器控制台检查：

```javascript
// 在 intentMapper.ts 中添加
console.log('[验证] 接收到的数据:', raw);
console.log('[验证] 映射后的数据:', mapped);
```

**预期结果**：
- ✅ 原始数据使用 camelCase
- ✅ 映射后的数据类型正确
- ✅ 不需要 snake_case 转换

### 验证点 3：前端发送数据

在 `DecisionList.tsx` 中检查：

```typescript
if (DEBUG) {
  console.log('[DecisionList] 发送给 Rust 的 qaPair:', selectedQaPair);
}
```

**预期结果**：
- ✅ 对象使用 camelCase
- ✅ 没有 snake_case 转换

### 验证点 4：Rust 反序列化输入

在 Rust 命令中添加调试日志：

```rust
// src-tauri/src/commands.rs
#[tauri::command]
pub async fn cmd_analyze_decision(
    qa_pair: DecisionQAPair,
    language: String,
) -> Result<DecisionAnalysis, CommandError> {
    #[cfg(debug_assertions)]
    {
        eprintln!("[cmd_analyze_decision] 接收到的 qa_pair:");
        eprintln!("  qa_index: {}", qa_pair.qa_index);
        eprintln!("  assistant_answer_uuid: {}", qa_pair.assistant_answer_uuid);
    }

    // ...
}
```

**预期结果**：
- ✅ 成功反序列化，无错误
- ✅ 字段值正确

---

## 🐛 常见问题排查

### 问题 1：编译错误

**错误**：`the trait bound DecisionQAPair: serde::Serialize is not satisfied`

**解决方案**：
1. 检查是否正确导入 `use serde::{Serialize, Deserialize};`
2. 确保 `#[serde(rename_all = "camelCase")]` 在 `#[ts(rename_all = "camelCase")]` 之前
3. 运行 `cargo clean` 清理构建缓存

### 问题 2：前端类型错误

**错误**：`Type 'OpeningIntent | null' is not assignable to type 'OpeningIntent'`

**解决方案**：
1. 检查 `intentMapper.ts` 中的 `mapOpeningIntent` 函数
2. 确保返回类型是 `OpeningIntent` 而不是 `OpeningIntent | null`
3. 使用默认值替代 `null`

### 问题 3：运行时错误

**错误**：`missing field 'qa_index'`

**解决方案**：
1. 确认 Rust 结构体有 `#[serde(rename_all = "camelCase")]`
2. 重新生成 TypeScript 类型：`cargo run --bin generate_types`
3. 重启应用

### 问题 4：字段名仍显示 snake_case

**原因**：数据库中存储的是旧格式

**解决方案**：
1. 删除旧的历史记录
2. 重新分析会话
3. 或者编写数据库迁移脚本

---

## 📊 测试检查清单

### 编译阶段
- [ ] `cargo check` 通过（无错误）
- [ ] `cargo run --bin generate_types` 成功
- [ ] `npm run build` 成功

### 运行阶段
- [ ] 应用启动成功
- [ ] 可以访问"会话意图分析"页面
- [ ] 可以加载测试会话

### 功能验证
- [ ] 左侧显示"开场白已分析"（非 N/A）
- [ ] 中间显示正确的问答对编号（非 #NaN）
- [ ] 点击问答对后右侧显示决策分析
- [ ] 无"missing field 'qa_index'"错误
- [ ] 控制台无类型错误

### 数据流验证
- [ ] Rust 序列化输出使用 camelCase
- [ ] 前端接收数据使用 camelCase
- [ ] 前端发送数据使用 camelCase
- [ ] Rust 反序列化成功

---

## 🎯 修复效果对比

### 修复前

**Rust**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(rename_all = "camelCase")]  // ❌ 只有 ts 属性
pub struct DecisionQAPair {
    pub qa_index: usize,
    // ...
}
```

**前端**：
```typescript
// ❌ 需要手动转换
function convertToSnakeCase(qaPair: DecisionQAPair): any {
  return {
    qa_index: qaPair.qaIndex,
    // ...
  };
}
```

**数据流**：
```
Rust → JSON: {"qa_index": 0}  (snake_case)
前端 TS: { qaIndex: number }  (camelCase) ❌ 不匹配
前端 → Rust: { qa_index: 0 }  (需要手动转换)
```

### 修复后

**Rust**：
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]  // ✅ 添加 serde 属性
#[ts(rename_all = "camelCase")]
pub struct DecisionQAPair {
    pub qa_index: usize,
    // ...
}
```

**前端**：
```typescript
// ✅ 直接传递，无需转换
const result = await invoke<DecisionAnalysis>('cmd_analyze_decision', {
  qaPair: selectedQaPair,  // 直接传递
  language,
});
```

**数据流**：
```
Rust → JSON: {"qaIndex": 0}  (camelCase) ✅
前端 TS: { qaIndex: number }  (camelCase) ✅ 匹配
前端 → Rust: { qaIndex: 0 }  (自动转换) ✅
```

---

## 📝 总结

本次修复实现了以下目标：

1. ✅ **根因修复**：添加 `#[serde(rename_all = "camelCase")]` 确保序列化一致性
2. ✅ **代码简化**：移除前端的 `convertToSnakeCase` 转换函数
3. ✅ **类型安全**：TypeScript 类型与 Rust 结构体完全匹配
4. ✅ **维护性**：统一使用 camelCase，减少混淆

**下次修改建议**：
- 所有新的 Rust 结构体都应同时添加 `#[serde(rename_all)]` 和 `#[ts(rename_all)]`
- 避免使用 snake_case 作为公开 API 的字段名
- 定期运行 `cargo run --bin generate_types` 同步类型定义
