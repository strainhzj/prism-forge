/**
 * PromptLibrary 组件
 *
 * 提示词库管理组件 - 用于管理和浏览保存的提示词
 * 支持三栏布局：Next Goals / AI Analysis / Meta Templates
 * 支持搜索、筛选、排序、编辑、删除等功能
 */

import { useState, useCallback, useEffect, useMemo } from 'react';
import { Search, Filter, ArrowUpDown, Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import {
  Tabs,
  TabsList
} from '@/components/ui/tabs';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { PromptCard } from '@/components/PromptCard';
import type {
  PromptLibraryItem,
  PromptCategory,
  PromptLibraryFilters
} from '@/types/prompt';

export interface PromptLibraryProps {
  /**
   * 初始选中的分类
   */
  defaultCategory?: PromptCategory;
  /**
   * 使用提示词回调
   */
  onUsePrompt?: (prompt: PromptLibraryItem) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * PromptLibrary 组件
 */
export function PromptLibrary({
  defaultCategory = 'next_goals',
  onUsePrompt,
  className
}: PromptLibraryProps) {
  // 状态管理
  const [activeTab, setActiveTab] = useState<PromptCategory>(defaultCategory);
  const [items, setItems] = useState<PromptLibraryItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 筛选状态
  const [filters, setFilters] = useState<PromptLibraryFilters>({
    search: '',
    category: 'all',
    minRating: 0,
    sortBy: 'created_at',
    sortOrder: 'desc'
  });

  // 编辑状态
  const [editingItem, setEditingItem] = useState<PromptLibraryItem | null>(null);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [editForm, setEditForm] = useState({
    title: '',
    content: ''
  });

  /**
   * 加载提示词列表
   */
  const loadItems = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      // TODO: 调用后端 API 获取提示词列表
      // 暂时使用空数组，等后端 API 实现后再调用
      setItems([]);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : '加载失败';
      setError(errorMessage);
      console.error('加载提示词库失败:', err);
    } finally {
      setLoading(false);
    }
  }, [activeTab]);

  /**
   * 初始化加载
   */
  useEffect(() => {
    loadItems();
  }, [loadItems]);

  /**
   * 应用筛选和排序
   */
  const processedItems = useMemo(() => {
    let result = [...items];

    // 搜索过滤
    if (filters.search) {
      const searchLower = filters.search.toLowerCase();
      result = result.filter((item) => {
        const title = 'title' in item ? item.title : item.name;
        const content = item.content.toLowerCase();
        return (
          title.toLowerCase().includes(searchLower) ||
          content.includes(searchLower)
        );
      });
    }

    // 分类过滤
    if (filters.category && filters.category !== 'all') {
      result = result.filter((item) => {
        if ('category' in item) {
          return item.category === filters.category;
        }
        return filters.category === 'meta_template';
      });
    }

    // 评分过滤
    if (filters.minRating && filters.minRating > 0) {
      result = result.filter((item) => {
        if ('rating' in item && item.rating) {
          return item.rating >= filters.minRating!;
        }
        return false;
      });
    }

    // 排序
    result.sort((a, b) => {
      const { sortBy, sortOrder } = filters;
      const multiplier = sortOrder === 'asc' ? 1 : -1;

      switch (sortBy) {
        case 'created_at':
          const aDate = 'created_at' in a ? new Date(a.created_at || 0).getTime() : 0;
          const bDate = 'created_at' in b ? new Date(b.created_at || 0).getTime() : 0;
          return (aDate - bDate) * multiplier;

        case 'usage_count':
          const aUsage = 'usage_count' in a ? a.usage_count : 0;
          const bUsage = 'usage_count' in b ? b.usage_count : 0;
          return (aUsage - bUsage) * multiplier;

        case 'rating':
          const aRating = 'rating' in a ? (a.rating || 0) : 0;
          const bRating = 'rating' in b ? (b.rating || 0) : 0;
          return (aRating - bRating) * multiplier;

        case 'title':
          const aTitle = 'title' in a ? a.title : a.name;
          const bTitle = 'title' in b ? b.title : b.name;
          return aTitle.localeCompare(bTitle) * multiplier;

        default:
          return 0;
      }
    });

    return result;
  }, [items, filters]);

  /**
   * 处理使用提示词
   */
  const handleUsePrompt = useCallback(
    (prompt: PromptLibraryItem) => {
      onUsePrompt?.(prompt);

      // 更新使用次数（后端 API 实现后调用）
      if (prompt.id) {
        // TODO: invoke('increment_prompt_usage', { id: prompt.id })
      }
    },
    [onUsePrompt]
  );

  /**
   * 处理编辑
   */
  const handleEdit = useCallback((prompt: PromptLibraryItem) => {
    setEditingItem(prompt);
    setEditForm({
      title: 'title' in prompt ? prompt.title : prompt.name,
      content: prompt.content
    });
    setEditDialogOpen(true);
  }, []);

  /**
   * 保存编辑
   */
  const handleSaveEdit = async () => {
    if (!editingItem) return;

    try {
      // TODO: 调用后端 API 保存修改
      setEditDialogOpen(false);
      await loadItems(); // 重新加载列表
    } catch (err) {
      console.error('保存失败:', err);
    }
  };

  /**
   * 处理删除
   */
  const handleDelete = async (_id: number) => {
    try {
      // TODO: 调用后端 API 删除
      await loadItems(); // 重新加载列表
    } catch (err) {
      console.error('删除失败:', err);
    }
  };

  /**
   * 处理评分变更
   */
  const handleRatingChange = async (_id: number, _rating: number) => {
    try {
      // TODO: 调用后端 API 更新评分
      await loadItems(); // 重新加载列表
    } catch (err) {
      console.error('更新评分失败:', err);
    }
  };

  /**
   * 重置筛选
   */
  const handleResetFilters = () => {
    setFilters({
      search: '',
      category: 'all',
      minRating: 0,
      sortBy: 'created_at',
      sortOrder: 'desc'
    });
  };

  /**
   * 获取分类标签
   */
  const getCategoryLabel = (category: PromptCategory): string => {
    const labels: Record<PromptCategory, string> = {
      next_goals: '用户目标',
      ai_analysis: 'AI 分析',
      meta_template: 'Meta 模板'
    };
    return labels[category];
  };

  /**
   * 渲染标签页内容
   */
  const renderTabContent = () => {
    // 加载状态
    if (loading) {
      return (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          <span className="ml-2 text-muted-foreground">加载中...</span>
        </div>
      );
    }

    // 错误状态
    if (error) {
      return (
        <Card className="p-8 text-center">
          <p className="text-destructive mb-2">加载失败</p>
          <p className="text-sm text-muted-foreground mb-4">{error}</p>
          <Button onClick={loadItems} variant="outline" size="sm">
            重试
          </Button>
        </Card>
      );
    }

    // 空状态
    if (processedItems.length === 0) {
      return (
        <Card className="p-12 text-center text-muted-foreground">
          <p className="text-lg font-medium mb-2">
            {filters.search || filters.category !== 'all' || filters.minRating! > 0
              ? '没有符合条件的提示词'
              : `暂无${getCategoryLabel(activeTab)}`}
          </p>
          <p className="text-sm">
            {filters.search || filters.category !== 'all' || filters.minRating! > 0
              ? '请调整筛选条件'
              : '提示词将在您生成并保存后显示在这里'}
          </p>
        </Card>
      );
    }

    // 提示词列表
    return (
      <>
        <div className="grid grid-cols-1 gap-3">
          {processedItems.map((item) => (
            <PromptCard
              key={item.id || ('key' in item ? item.key : String(item.id))}
              prompt={item}
              showRatingSelector
              onRatingChange={handleRatingChange}
              onEdit={handleEdit}
              onDelete={handleDelete}
              onUse={handleUsePrompt}
            />
          ))}
        </div>

        {/* 统计信息 */}
        <div className="text-center text-sm text-muted-foreground">
          显示 {processedItems.length} 个提示词
        </div>
      </>
    );
  };

  return (
    <div className={cn('space-y-4', className)}>
      {/* 工具栏：搜索和筛选 */}
      <Card className="p-4">
        <div className="space-y-4">
          {/* 搜索框 */}
          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="搜索提示词标题或内容..."
              value={filters.search}
              onChange={(e) =>
                setFilters((prev) => ({ ...prev, search: e.target.value }))
              }
              className="pl-9"
            />
          </div>

          {/* 筛选和排序 */}
          <div className="flex items-center gap-3 flex-wrap">
            {/* 分类筛选 */}
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-muted-foreground" />
              <Select
                value={filters.category}
                onValueChange={(value: string) =>
                  setFilters((prev) => ({
                    ...prev,
                    category: value as PromptCategory | 'all'
                  }))
                }
              >
                <SelectTrigger className="w-[140px]">
                  <SelectValue placeholder="全部分类" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部分类</SelectItem>
                  <SelectItem value="next_goals">用户目标</SelectItem>
                  <SelectItem value="ai_analysis">AI 分析</SelectItem>
                  <SelectItem value="meta_template">Meta 模板</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* 评分筛选 */}
            <Select
              value={filters.minRating?.toString() || '0'}
              onValueChange={(value: string) =>
                setFilters((prev) => ({ ...prev, minRating: parseInt(value) }))
              }
            >
              <SelectTrigger className="w-[120px]">
                <SelectValue placeholder="最低评分" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0">全部评分</SelectItem>
                <SelectItem value="4">4 星以上</SelectItem>
                <SelectItem value="3">3 星以上</SelectItem>
                <SelectItem value="2">2 星以上</SelectItem>
                <SelectItem value="1">1 星以上</SelectItem>
              </SelectContent>
            </Select>

            {/* 排序 */}
            <div className="flex items-center gap-2">
              <ArrowUpDown className="h-4 w-4 text-muted-foreground" />
              <Select
                value={`${filters.sortBy}-${filters.sortOrder}`}
                onValueChange={(value: string) => {
                  const [sortBy, sortOrder] = value.split('-');
                  setFilters((prev) => ({
                    ...prev,
                    sortBy: sortBy as PromptLibraryFilters['sortBy'],
                    sortOrder: sortOrder as 'asc' | 'desc'
                  }));
                }}
              >
                <SelectTrigger className="w-[140px]">
                  <SelectValue placeholder="排序方式" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="created_at-desc">最新创建</SelectItem>
                  <SelectItem value="created_at-asc">最早创建</SelectItem>
                  <SelectItem value="usage_count-desc">使用最多</SelectItem>
                  <SelectItem value="usage_count-asc">使用最少</SelectItem>
                  <SelectItem value="rating-desc">评分最高</SelectItem>
                  <SelectItem value="rating-asc">评分最低</SelectItem>
                  <SelectItem value="title-asc">标题 A-Z</SelectItem>
                  <SelectItem value="title-desc">标题 Z-A</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* 重置按钮 */}
            <Button
              variant="ghost"
              size="sm"
              onClick={handleResetFilters}
              className="ml-auto"
            >
              重置筛选
            </Button>
          </div>
        </div>
      </Card>

      {/* 三栏标签页 */}
      <Tabs>
        <TabsList className="w-full justify-start">
          <button
            onClick={() => setActiveTab('next_goals')}
            className={cn(
              'inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
              activeTab === 'next_goals'
                ? 'bg-background text-foreground shadow'
                : 'text-muted-foreground'
            )}
          >
            用户目标
          </button>
          <button
            onClick={() => setActiveTab('ai_analysis')}
            className={cn(
              'inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
              activeTab === 'ai_analysis'
                ? 'bg-background text-foreground shadow'
                : 'text-muted-foreground'
            )}
          >
            AI 分析
          </button>
          <button
            onClick={() => setActiveTab('meta_template')}
            className={cn(
              'inline-flex items-center justify-center whitespace-nowrap rounded-md px-3 py-1 text-sm font-medium ring-offset-background transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
              activeTab === 'meta_template'
                ? 'bg-background text-foreground shadow'
                : 'text-muted-foreground'
            )}
          >
            Meta 模板
          </button>
        </TabsList>

        {/* 标签页内容 */}
        <div className="space-y-4 mt-2">{renderTabContent()}</div>
      </Tabs>

      {/* 编辑对话框 */}
      <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>编辑提示词</DialogTitle>
            <DialogDescription>修改提示词的标题和内容</DialogDescription>
          </DialogHeader>

          <div className="space-y-4">
            <div>
              <Label htmlFor="edit-title">标题</Label>
              <Input
                id="edit-title"
                value={editForm.title}
                onChange={(e) =>
                  setEditForm((prev) => ({ ...prev, title: e.target.value }))
                }
                placeholder="输入提示词标题"
              />
            </div>

            <div>
              <Label htmlFor="edit-content">内容</Label>
              <Textarea
                id="edit-content"
                value={editForm.content}
                onChange={(e) =>
                  setEditForm((prev) => ({ ...prev, content: e.target.value }))
                }
                placeholder="输入提示词内容"
                rows={12}
                className="font-mono text-sm"
              />
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setEditDialogOpen(false)}>
              取消
            </Button>
            <Button onClick={handleSaveEdit}>保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
