/**
 * SessionStatsSidebar 组件
 *
 * 会话详情页右侧边栏，显示会话统计信息和元数据
 * 支持深浅色主题
 */

import { memo, useMemo } from 'react';
import {
  MessageSquare,
  Coins,
  Clock,
  Tag,
  Star,
  Folder,
  Calendar,
  HardDrive,
  FileText,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SessionStatsSidebar] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

export interface SessionStatsSidebarProps {
  /**
   * 会话ID
   */
  sessionId: string;
  /**
   * 项目名称
   */
  projectName: string;
  /**
   * 项目路径
   */
  projectPath?: string;
  /**
   * 会话评分（1-5星）
   */
  rating?: number | null;
  /**
   * 会话标签（JSON 数组字符串）
   */
  tags?: string;
  /**
   * 创建时间（ISO 8601 格式）
   */
  createdAt?: string;
  /**
   * 更新时间（ISO 8601 格式）
   */
  updatedAt?: string;
  /**
   * Token 统计信息
   */
  tokenStats?: {
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
    messageCount: number;
  };
  /**
   * 文件大小（字节）
   */
  fileSize?: number;
  /**
   * 消息总数
   */
  messageCount?: number;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 格式化文件大小
 */
function formatFileSize(bytes?: number): string {
  if (!bytes) return '未知';

  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

/**
 * 格式化日期时间
 */
function formatDateTime(dateString?: string): string {
  if (!dateString) return '未知';

  try {
    const date = new Date(dateString);
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return dateString;
  }
}

/**
 * 解析标签 JSON 字符串
 */
function parseTags(tagsJson?: string): string[] {
  if (!tagsJson || tagsJson === '[]') {
    return [];
  }
  try {
    return JSON.parse(tagsJson) as string[];
  } catch {
    return [];
  }
}

/**
 * 计算Token成本估算（使用 gpt-4o-mini 定价）
 */
function estimateCost(tokens: number): string {
  // gpt-4o-mini 定价（截至2025年1月）
  const inputPrice = 0.15 / 1_000_000; // $0.15 per 1M tokens
  const outputPrice = 0.60 / 1_000_000; // $0.60 per 1M tokens

  // 简化估算：假设 60% 输入，40% 输出
  const cost = tokens * (inputPrice * 0.6 + outputPrice * 0.4);

  if (cost < 0.01) return '< $0.01';
  return `$${cost.toFixed(4)}`;
}

/**
 * StatsItem 组件 - 统计项
 */
interface StatsItemProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  className?: string;
}

const StatsItem = memo(function StatsItem({
  icon,
  label,
  value,
  className,
}: StatsItemProps) {
  return (
    <div className={cn('flex items-center gap-2 text-sm', className)}>
      <div className="text-gray-400 flex-shrink-0">
        {icon}
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-gray-400 text-xs">{label}</div>
        <div className="font-medium truncate text-white">{value}</div>
      </div>
    </div>
  );
});

/**
 * SessionStatsSidebar 组件
 *
 * @example
 * <SessionStatsSidebar
 *   sessionId="xxx"
 *   projectName="My Project"
 *   rating={5}
 *   tags='["tag1", "tag2"]'
 *   tokenStats={{ inputTokens: 1000, outputTokens: 500, totalTokens: 1500, messageCount: 10 }}
 * />
 */
export const SessionStatsSidebar = memo(function SessionStatsSidebar({
  sessionId,
  projectName,
  projectPath: _projectPath, // 标记为未使用
  rating,
  tags,
  createdAt,
  updatedAt,
  tokenStats,
  fileSize,
  messageCount,
  className,
}: SessionStatsSidebarProps) {
  const parsedTags = useMemo(() => parseTags(tags), [tags]);
  const formattedFileSize = useMemo(() => formatFileSize(fileSize), [fileSize]);
  const formattedCreatedAt = useMemo(() => formatDateTime(createdAt), [createdAt]);
  const formattedUpdatedAt = useMemo(() => formatDateTime(updatedAt), [updatedAt]);
  const estimatedCost = useMemo(
    () => (tokenStats ? estimateCost(tokenStats.totalTokens) : null),
    [tokenStats]
  );

  debugLog('render', { sessionId, projectName, rating, messageCount });

  return (
    <div className={cn('flex flex-col gap-4', className)}>
      {/* 会话信息卡片 */}
      <Card className="border-[#333]" style={{ backgroundColor: '#1E1E1E', borderColor: '#333' }}>
        <CardHeader className="pb-3">
          <CardTitle className="text-base font-semibold flex items-center gap-2 text-white">
            <FileText className="w-4 h-4" style={{ color: '#FF6B6B' }} />
            会话信息
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {/* 项目名称 */}
          <StatsItem
            icon={<Folder className="w-4 h-4" />}
            label="项目"
            value={projectName}
          />

          {/* 会话 ID */}
          <StatsItem
            icon={<Tag className="w-4 h-4" />}
            label="会话 ID"
            value={sessionId.slice(0, 12) + '...'}
          />

          {/* 评分 */}
          {rating !== null && rating !== undefined && (
            <div className="flex items-center gap-2 text-sm">
              <div className="text-gray-400 flex-shrink-0">
                <Star className="w-4 h-4" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-gray-400 text-xs">评分</div>
                <div className="font-medium flex items-center gap-1 text-white">
                  {Array.from({ length: 5 }).map((_, i) => (
                    <Star
                      key={i}
                      className={cn(
                        'w-3.5 h-3.5',
                        i < rating
                          ? 'fill-yellow-400 text-yellow-400'
                          : 'text-gray-600'
                      )}
                    />
                  ))}
                  <span className="ml-1">({rating}/5)</span>
                </div>
              </div>
            </div>
          )}

          {/* 标签 */}
          {parsedTags.length > 0 && (
            <div className="space-y-1.5">
              <div className="flex items-center gap-2 text-xs text-gray-400">
                <Tag className="w-3.5 h-3.5" />
                <span>标签</span>
              </div>
              <div className="flex flex-wrap gap-1.5">
                {parsedTags.map((tag, index) => {
                  const isOrange = index % 2 === 0;
                  return (
                    <Badge
                      key={tag}
                      className="text-xs"
                      style={{
                        backgroundColor: isOrange ? '#FF6B6B' : '#4A9EFF',
                        color: 'white',
                        boxShadow: `0 0 10px ${isOrange ? 'rgba(255, 107, 107, 0.3)' : 'rgba(74, 158, 255, 0.3)'}`
                      }}
                    >
                      {tag}
                    </Badge>
                  );
                })}
              </div>
            </div>
          )}

          <Separator className="my-2" style={{ backgroundColor: '#333' }} />

          {/* 创建时间 */}
          <StatsItem
            icon={<Calendar className="w-4 h-4" />}
            label="创建时间"
            value={formattedCreatedAt}
          />

          {/* 更新时间 */}
          <StatsItem
            icon={<Clock className="w-4 h-4" />}
            label="最后更新"
            value={formattedUpdatedAt}
          />

          {/* 文件大小 */}
          {fileSize && (
            <StatsItem
              icon={<HardDrive className="w-4 h-4" />}
              label="文件大小"
              value={formattedFileSize}
            />
          )}
        </CardContent>
      </Card>

      {/* 统计信息卡片 */}
      {tokenStats && (
        <Card className="border-[#333]" style={{ backgroundColor: '#1E1E1E', borderColor: '#333' }}>
          <CardHeader className="pb-3">
            <CardTitle className="text-base font-semibold flex items-center gap-2 text-white">
              <MessageSquare className="w-4 h-4" style={{ color: '#4A9EFF' }} />
              统计信息
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {/* 消息总数 */}
            <StatsItem
              icon={<MessageSquare className="w-4 h-4" />}
              label="消息总数"
              value={tokenStats.messageCount || messageCount || 0}
            />

            {/* Token 总数 */}
            <StatsItem
              icon={<Coins className="w-4 h-4" />}
              label="Token 总数"
              value={tokenStats.totalTokens.toLocaleString()}
            />

            {/* 输入 Token */}
            <div className="flex items-center gap-2 text-sm">
              <div className="text-gray-400 flex-shrink-0">
                <Coins className="w-4 h-4" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-gray-400 text-xs">输入 Token</div>
                <div className="font-medium text-white">
                  {tokenStats.inputTokens.toLocaleString()}
                </div>
                <div className="text-xs text-gray-400">
                  {((tokenStats.inputTokens / tokenStats.totalTokens) * 100).toFixed(1)}%
                </div>
              </div>
            </div>

            {/* 输出 Token */}
            <div className="flex items-center gap-2 text-sm">
              <div className="text-gray-400 flex-shrink-0">
                <Coins className="w-4 h-4" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="text-gray-400 text-xs">输出 Token</div>
                <div className="font-medium text-white">
                  {tokenStats.outputTokens.toLocaleString()}
                </div>
                <div className="text-xs text-gray-400">
                  {((tokenStats.outputTokens / tokenStats.totalTokens) * 100).toFixed(1)}%
                </div>
              </div>
            </div>

            {/* 成本估算 */}
            {estimatedCost && (
              <>
                <Separator className="my-2" style={{ backgroundColor: '#333' }} />
                <StatsItem
                  icon={<Coins className="w-4 h-4 text-yellow-400" />}
                  label="成本估算"
                  value={estimatedCost}
                />
              </>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
});
