/**
 * PromptLab 页面
 *
 * 提示词实验室 - 用于生成和管理优化的 AI 提示词
 * 提供双栏布局：Builder + Library
 */

import { useState } from 'react';
import { Wand2, BookMarked, Sparkles } from 'lucide-react';
import { PromptBuilder } from '@/components/PromptBuilder';
import { PromptLibrary } from '@/components/PromptLibrary';
import { ThemeToggle } from '@/components/ThemeToggle';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import type { EnhancedPrompt, PromptLibraryItem } from '@/types/prompt';

/**
 * PromptLab 页面
 */
export function PromptLab() {
  const [generatedPrompts, setGeneratedPrompts] = useState<EnhancedPrompt[]>([]);

  /**
   * 处理生成完成
   */
  const handleGenerated = (result: EnhancedPrompt) => {
    setGeneratedPrompts((prev) => [result, ...prev]);
  };

  /**
   * 处理使用库中的提示词
   */
  const handleUseLibraryPrompt = (prompt: PromptLibraryItem) => {
    console.log('使用提示词:', prompt);
    // TODO: 将提示词内容复制到剪贴板或填入输入框
  };

  return (
    <div className="container mx-auto py-6 max-w-7xl">
      {/* 页面头部 */}
      <div className="mb-6">
        <div className="flex items-center justify-between mb-2">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 rounded-lg">
              <Wand2 className="h-6 w-6 text-primary" />
            </div>
            <div>
              <h1 className="text-3xl font-bold tracking-tight text-foreground">提示词实验室</h1>
              <p className="text-muted-foreground mt-1">
                基于 AI 和历史会话生成优化的提示词，提高开发效率
              </p>
            </div>
          </div>
          <ThemeToggle />
        </div>
      </div>

      {/* 主内容 */}
      <Tabs defaultValue="builder" className="space-y-4">
        <TabsList className="w-full justify-start">
          <TabsTrigger value="builder" className="flex items-center gap-2">
            <Wand2 className="h-4 w-4" />
            生成器
          </TabsTrigger>
          <TabsTrigger value="library" className="flex items-center gap-2">
            <BookMarked className="h-4 w-4" />
            提示词库
          </TabsTrigger>
        </TabsList>

        {/* 生成器标签页 */}
        <TabsContent value="builder" className="space-y-4">
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* 主生成区域 */}
            <div className="lg:col-span-2">
              <div className="space-y-4">
                {/* 提示词构建器 */}
                <div>
                  <h2 className="text-lg font-semibold mb-3">创建新提示词</h2>
                  <PromptBuilder onGenerated={handleGenerated} />
                </div>

                {/* 历史记录 */}
                {generatedPrompts.length > 0 && (
                  <div>
                    <h2 className="text-lg font-semibold mb-3">最近生成</h2>
                    <div className="space-y-3">
                      {generatedPrompts.slice(0, 3).map((prompt, index) => (
                        <GeneratedPromptCard
                          key={`${prompt.originalGoal.slice(0, 20)}-${index}`}
                          prompt={prompt}
                        />
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* 侧边栏 - 提示和技巧 */}
            <div className="space-y-4">
              {/* 使用指南 */}
              <Card className="p-4">
                <h3 className="font-semibold mb-3 flex items-center gap-2">
                  <Sparkles className="h-4 w-4 text-primary" />
                  使用指南
                </h3>
                <div className="space-y-3 text-sm text-muted-foreground">
                  <div>
                    <p className="font-medium text-foreground mb-1">1. 描述目标</p>
                    <p className="text-xs">
                      清晰地描述您要完成的任务，越具体越好
                    </p>
                  </div>
                  <div>
                    <p className="font-medium text-foreground mb-1">2. AI 分析</p>
                    <p className="text-xs">
                      系统会检索相关历史会话并分析最佳实践
                    </p>
                  </div>
                  <div>
                    <p className="font-medium text-foreground mb-1">3. 生成提示词</p>
                    <p className="text-xs">
                      获得结构化、优化的提示词，可直接用于 AI 编程助手
                    </p>
                  </div>
                  <div>
                    <p className="font-medium text-foreground mb-1">4. 复制使用</p>
                    <p className="text-xs">
                      一键复制或保存到提示词库供后续使用
                    </p>
                  </div>
                </div>
              </Card>

              {/* 示例 */}
              <Card className="p-4">
                <h3 className="font-semibold mb-3">示例目标</h3>
                <div className="space-y-2">
                  <ExampleButton
                    text="实现用户登录功能"
                    onClick={() => {
                      // TODO: 自动填充到输入框
                    }}
                  />
                  <ExampleButton
                    text="创建 REST API 端点"
                    onClick={() => {}}
                  />
                  <ExampleButton
                    text="添加数据库迁移脚本"
                    onClick={() => {}}
                  />
                  <ExampleButton
                    text="实现文件上传组件"
                    onClick={() => {}}
                  />
                </div>
              </Card>

              {/* 统计信息 */}
              {generatedPrompts.length > 0 && (
                <Card className="p-4">
                  <h3 className="font-semibold mb-3">本次会话统计</h3>
                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">生成次数</span>
                      <span className="font-medium">{generatedPrompts.length}</span>
                    </div>
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">平均节省</span>
                      <span className="font-medium">
                        {(
                          generatedPrompts.reduce(
                            (sum, p) => sum + p.tokenStats.savingsPercentage,
                            0
                          ) / generatedPrompts.length
                        ).toFixed(1)}
                        %
                      </span>
                    </div>
                  </div>
                </Card>
              )}
            </div>
          </div>
        </TabsContent>

        {/* 提示词库标签页 */}
        <TabsContent value="library">
          <PromptLibrary onUsePrompt={handleUseLibraryPrompt} />
        </TabsContent>
      </Tabs>
    </div>
  );
}

/**
 * 生成的提示词卡片
 */
interface GeneratedPromptCardProps {
  prompt: EnhancedPrompt;
}

function GeneratedPromptCard({ prompt }: GeneratedPromptCardProps) {
  /**
   * 复制提示词
   */
  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(prompt.enhancedPrompt);
    } catch (err) {
      console.error('复制失败:', err);
    }
  };

  return (
    <Card className="p-4 hover:bg-accent/50 transition-colors">
      <div className="flex items-start justify-between mb-2">
        <div className="flex-1 min-w-0">
          <p className="font-medium text-sm truncate mb-1">
            {prompt.originalGoal}
          </p>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span>{prompt.tokenStats.compressedTokens} tokens</span>
            {prompt.referencedSessions.length > 0 && (
              <>
                <span>•</span>
                <span>{prompt.referencedSessions.length} 个会话</span>
              </>
            )}
          </div>
        </div>
        <Button
          variant="ghost"
          size="sm"
          onClick={handleCopy}
          className="shrink-0 h-7 px-2"
        >
          复制
        </Button>
      </div>
    </Card>
  );
}

/**
 * 示例按钮
 */
interface ExampleButtonProps {
  text: string;
  onClick: () => void;
}

function ExampleButton({ text, onClick }: ExampleButtonProps) {
  return (
    <button
      onClick={onClick}
      className="w-full text-left px-3 py-2 text-sm bg-muted/30 hover:bg-muted/50 rounded-md transition-colors"
    >
      {text}
    </button>
  );
}
