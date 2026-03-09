/**
 * 意图分析数据映射工具
 *
 * 解决 Rust snake_case 和 TypeScript camelCase 之间的字段名不匹配问题
 */

import type { OpeningIntent } from '@/types/generated/OpeningIntent';
import type { IntentAnalysisHistory } from '@/types/generated/IntentAnalysisHistory';
import type { DecisionQAPair } from '@/types/generated';

/**
 * 从后端原始数据映射 OpeningIntent
 *
 * 后端使用 snake_case，前端使用 camelCase
 * 需要手动映射字段名
 */
export function mapOpeningIntent(raw: any): OpeningIntent | null {
  if (!raw) return null;

  // 尝试直接访问 camelCase 字段（如果已正确映射）
  if (raw.intentType && raw.confidence !== undefined) {
    return raw as OpeningIntent;
  }

  // 手动映射 snake_case 到 camelCase
  return {
    intentType: raw.intent_type || '',
    confidence: raw.confidence ?? 0,
    description: raw.description ?? null,
    keyInfo: raw.key_info ?? [],
  };
}

/**
 * 从后端原始数据映射 IntentAnalysisHistory
 */
export function mapIntentAnalysisHistory(raw: any): IntentAnalysisHistory | null {
  if (!raw) return null;

  return {
    id: raw.id ?? BigInt(0),
    sessionFilePath: raw.session_file_path || raw.sessionFilePath || '',
    qaPairs: raw.qa_pairs || raw.qaPairs || [],
    openingIntent: mapOpeningIntent(raw.opening_intent || raw.openingIntent),
    language: raw.language || '',
    analyzedAt: raw.analyzed_at || raw.analyzedAt || '',
    createdAt: raw.created_at || raw.createdAt || '',
  };
}

/**
 * 从后端原始数据映射 DecisionQAPair
 */
export function mapDecisionQAPair(raw: any): DecisionQAPair {
  return {
    qaIndex: raw.qa_index ?? raw.qaIndex ?? 0,
    assistantAnswerUuid: raw.assistant_answer_uuid || raw.assistantAnswerUuid || '',
    userDecisionUuid: raw.user_decision_uuid || raw.userDecisionUuid || '',
    assistantAnswer: raw.assistant_answer || raw.assistantAnswer || '',
    userDecision: raw.user_decision || raw.userDecision || '',
    contextQaPairs: raw.context_qa_pairs || raw.contextQaPairs || undefined,
  };
}

/**
 * 批量映射 DecisionQAPair 数组
 */
export function mapDecisionQAPairs(rawArray: any[]): DecisionQAPair[] {
  if (!Array.isArray(rawArray)) return [];
  return rawArray.map(mapDecisionQAPair);
}
