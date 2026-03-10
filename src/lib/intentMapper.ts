/**
 * 意图分析数据映射工具
 *
 * 解决 Rust snake_case 和 TypeScript camelCase 之间的字段名不匹配问题
 */

import type { OpeningIntent } from '@/types/generated/OpeningIntent';
import type { IntentAnalysisHistory } from '@/types/generated/IntentAnalysisHistory';
import type { DecisionQAPair } from '@/types/generated';

/**
 * 默认的 OpeningIntent 值（用于 null/undefined 情况）
 */
const DEFAULT_OPENING_INTENT: OpeningIntent = {
  intentType: '',
  confidence: 0,
  description: null,
  keyInfo: [],
};

/**
 * 从后端原始数据映射 OpeningIntent
 *
 * Rust 使用 #[serde(rename_all = "camelCase")] 和 #[ts(rename_all = "camelCase")]
 * 所以返回的 JSON 字段名应该是 camelCase
 *
 * 但为了向后兼容，我们也支持 snake_case
 */
export function mapOpeningIntent(raw: any): OpeningIntent {
  if (!raw) return DEFAULT_OPENING_INTENT;

  // 🔧 修复：优先检查 camelCase 字段（Rust 序列化的标准输出）
  if (raw.intentType !== undefined && raw.confidence !== undefined) {
    // ✅ 已经是 camelCase 格式，直接返回
    return raw as OpeningIntent;
  }

  // 🔧 修复：如果 camelCase 不存在，尝试手动映射 snake_case
  if (raw.intent_type !== undefined || raw.confidence !== undefined || raw.description !== undefined || raw.key_info !== undefined) {
    // ✅ 存在 snake_case 字段，手动映射
    const mapped: OpeningIntent = {
      intentType: raw.intent_type || '',
      confidence: raw.confidence ?? 0,
      description: raw.description ?? null,
      keyInfo: raw.key_info ?? [],
    };

    // 🔍 调试日志（仅开发环境）
    if (import.meta.env.DEV) {
      console.log('[mapOpeningIntent] 从 snake_case 映射到 camelCase:', {
        input: raw,
        output: mapped,
      });
    }

    return mapped;
  }

  // ❌ 既没有 camelCase 也没有 snake_case，返回默认值
  if (import.meta.env.DEV) {
    console.warn('[mapOpeningIntent] 无效的 openingIntent 数据，使用默认值:', raw);
  }

  return DEFAULT_OPENING_INTENT;
}

/**
 * 从后端原始数据映射 IntentAnalysisHistory
 *
 * Rust 使用 #[serde(rename_all = "camelCase")] 和 #[ts(rename_all = "camelCase")]
 * 所以返回的 JSON 字段名应该是 camelCase
 *
 * 但为了向后兼容，我们也支持 snake_case
 */
export function mapIntentAnalysisHistory(raw: any): IntentAnalysisHistory | null {
  if (!raw) return null;

  // 🔧 修复：优先使用 camelCase 字段（Rust 序列化的标准输出）
  const openingIntentRaw = raw.openingIntent || raw.opening_intent;
  const qaPairsRaw = raw.qaPairs || raw.qa_pairs;
  const sessionFilePath = raw.sessionFilePath || raw.session_file_path || '';
  const analyzedAt = raw.analyzedAt || raw.analyzed_at || '';
  const createdAt = raw.createdAt || raw.created_at || '';

  // 🔧 修复：使用 mapDecisionQAPairs 映射数组元素
  const mappedQaPairs = mapDecisionQAPairs(qaPairsRaw);

  // ✅ mapOpeningIntent 现在总是返回非空值
  const mapped: IntentAnalysisHistory = {
    id: raw.id ?? BigInt(0),
    sessionFilePath,
    qaPairs: mappedQaPairs,
    openingIntent: mapOpeningIntent(openingIntentRaw),
    language: raw.language || '',
    analyzedAt,
    createdAt,
  };

  // 🔍 调试日志（仅开发环境）
  if (import.meta.env.DEV) {
    console.log('[mapIntentAnalysisHistory] 映射完成:', {
      input: raw,
      output: {
        ...mapped,
        openingIntent: mapped.openingIntent,
      },
      hasOpeningIntent: !!mapped.openingIntent,
      intentType: mapped.openingIntent?.intentType,
      qaPairsCount: mapped.qaPairs.length,
      firstQaPairIndex: mapped.qaPairs[0]?.qaIndex,
    });
  }

  return mapped;
}

/**
 * 从后端原始数据映射 DecisionQAPair
 *
 * Rust 使用 #[ts(rename_all = "camelCase")] 序列化
 * 所以返回的 JSON 字段名应该是 camelCase
 *
 * 但为了兼容性，我们也支持 snake_case
 */
export function mapDecisionQAPair(raw: any): DecisionQAPair {
  // 🔧 修复：优先使用 camelCase 字段（Rust 序列化的标准输出）
  const qaIndex = raw.qaIndex ?? raw.qa_index ?? 0;
  const assistantAnswerUuid = raw.assistantAnswerUuid ?? raw.assistant_answer_uuid ?? '';
  const userDecisionUuid = raw.userDecisionUuid ?? raw.user_decision_uuid ?? '';
  const assistantAnswer = raw.assistantAnswer ?? raw.assistant_answer ?? '';
  const userDecision = raw.userDecision ?? raw.user_decision ?? '';
  const contextQaPairs = raw.contextQaPairs ?? raw.context_qa_pairs ?? undefined;

  // 🔍 调试日志（仅开发环境）
  if (import.meta.env.DEV && (qaIndex === 0 || !assistantAnswerUuid)) {
    console.warn('[mapDecisionQAPair] 可能存在字段映射问题:', {
      input: raw,
      output: { qaIndex, assistantAnswerUuid },
    });
  }

  return {
    qaIndex,
    assistantAnswerUuid,
    userDecisionUuid,
    assistantAnswer,
    userDecision,
    contextQaPairs,
  };
}

/**
 * 批量映射 DecisionQAPair 数组
 */
export function mapDecisionQAPairs(rawArray: any[]): DecisionQAPair[] {
  if (!Array.isArray(rawArray)) return [];
  return rawArray.map(mapDecisionQAPair);
}
