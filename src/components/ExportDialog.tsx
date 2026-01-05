/**
 * ExportDialog 组件
 *
 * 导出对话框 - 用于配置和执行会话数据导出
 * 支持多种格式和选项
 */

import { useState, useCallback } from 'react';
import { Download, FileJson, FileSpreadsheet, FileText, Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog';
import type { ExportData, ExportFormat, ExportOptions } from '@/types/export';
import { exportData, triggerDownload, getFormatDescription } from '@/utils/exporters';

export interface ExportDialogProps {
  /**
   * 是否显示对话框
   */
  open: boolean;
  /**
   * 关闭对话框回调
   */
  onOpenChange: (open: boolean) => void;
  /**
   * 要导出的数据
   */
  data: ExportData;
  /**
   * 导出完成回调
   */
  onExportComplete?: (filename: string) => void;
  /**
   * 自定义类名
   */
  className?: string;
}

/**
 * 格式选项
 */
interface FormatOption {
  value: ExportFormat;
  label: string;
  icon: React.ReactNode;
  description: string;
}

/**
 * 格式选项列表
 */
const FORMAT_OPTIONS: FormatOption[] = [
  {
    value: 'json',
    label: 'JSON',
    icon: <FileJson className="h-5 w-5" />,
    description: '完整的 JSON 格式，包含所有数据和元数据'
  },
  {
    value: 'csv',
    label: 'CSV',
    icon: <FileSpreadsheet className="h-5 w-5" />,
    description: '表格格式，适合在 Excel 等工具中查看'
  },
  {
    value: 'markdown',
    label: 'Markdown',
    icon: <FileText className="h-5 w-5" />,
    description: '可读的 Markdown 文档格式'
  }
];

/**
 * ExportDialog 组件
 */
export function ExportDialog({
  open,
  onOpenChange,
  data,
  onExportComplete,
  className
}: ExportDialogProps) {
  // 状态管理
  const [format, setFormat] = useState<ExportFormat>('json');
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includeCodeBlocks, setIncludeCodeBlocks] = useState(true);
  const [includeTimestamps, setIncludeTimestamps] = useState(true);
  const [csvDelimiter, setCsvDelimiter] = useState<',' | ';' | '\t'>(',');
  const [markdownHeadingLevel, setMarkdownHeadingLevel] = useState<1 | 2 | 3>(1);
  const [isExporting, setIsExporting] = useState(false);

  /**
   * 重置为默认选项
   */
  const resetOptions = useCallback(() => {
    setFormat('json');
    setIncludeMetadata(true);
    setIncludeCodeBlocks(true);
    setIncludeTimestamps(true);
    setCsvDelimiter(',');
    setMarkdownHeadingLevel(1);
  }, []);

  /**
   * 处理导出
   */
  const handleExport = useCallback(async () => {
    setIsExporting(true);

    try {
      const options: ExportOptions = {
        format,
        includeMetadata,
        includeCodeBlocks,
        includeTimestamps,
        csvDelimiter,
        markdownHeadingLevel
      };

      // 模拟异步处理（实际导出是同步的，但为了更好的用户体验）
      await new Promise((resolve) => setTimeout(resolve, 500));

      const result = exportData(data, options);
      triggerDownload(result);

      onExportComplete?.(result.filename);

      // 关闭对话框
      setTimeout(() => {
        onOpenChange(false);
        resetOptions();
      }, 500);
    } catch (error) {
      console.error('导出失败:', error);
    } finally {
      setIsExporting(false);
    }
  }, [
    format,
    includeMetadata,
    includeCodeBlocks,
    includeTimestamps,
    csvDelimiter,
    markdownHeadingLevel,
    data,
    onOpenChange,
    onExportComplete,
    resetOptions
  ]);

  /**
   * 处理取消
   */
  const handleCancel = () => {
    onOpenChange(false);
    resetOptions();
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className={cn('max-w-md', className)}>
        <DialogHeader>
          <DialogTitle>导出会话数据</DialogTitle>
          <DialogDescription>
            选择导出格式和选项来保存会话数据
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6">
          {/* 格式选择 */}
          <div className="space-y-3">
            <Label>导出格式</Label>
            <div className="grid grid-cols-3 gap-3">
              {FORMAT_OPTIONS.map((option) => (
                <button
                  key={option.value}
                  onClick={() => setFormat(option.value)}
                  className={cn(
                    'flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-colors',
                    format === option.value
                      ? 'border-primary bg-primary/5'
                      : 'border-muted hover:bg-muted/50'
                  )}
                >
                  {option.icon}
                  <span className="text-sm font-medium">{option.label}</span>
                </button>
              ))}
            </div>
            <p className="text-xs text-muted-foreground">
              {getFormatDescription(format)}
            </p>
          </div>

          {/* 导出选项 */}
          <div className="space-y-4">
            <Label>导出选项</Label>

            {/* 元数据 */}
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <label
                  htmlFor="include-metadata"
                  className="text-sm font-medium cursor-pointer"
                >
                  包含元数据
                </label>
                <p className="text-xs text-muted-foreground">
                  会话信息、统计等元数据
                </p>
              </div>
              <Checkbox
                id="include-metadata"
                checked={includeMetadata}
                onCheckedChange={(checked) =>
                  setIncludeMetadata(checked as boolean)
                }
              />
            </div>

            {/* 代码块 */}
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <label
                  htmlFor="include-codeblocks"
                  className="text-sm font-medium cursor-pointer"
                >
                  包含代码块
                </label>
                <p className="text-xs text-muted-foreground">
                  消息中的代码块内容
                </p>
              </div>
              <Checkbox
                id="include-codeblocks"
                checked={includeCodeBlocks}
                onCheckedChange={(checked) =>
                  setIncludeCodeBlocks(checked as boolean)
                }
              />
            </div>

            {/* 时间戳 */}
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <label
                  htmlFor="include-timestamps"
                  className="text-sm font-medium cursor-pointer"
                >
                  包含时间戳
                </label>
                <p className="text-xs text-muted-foreground">
                  每条消息的时间信息
                </p>
              </div>
              <Checkbox
                id="include-timestamps"
                checked={includeTimestamps}
                onCheckedChange={(checked) =>
                  setIncludeTimestamps(checked as boolean)
                }
              />
            </div>

            {/* 格式特定选项 */}
            {format === 'csv' && (
              <div className="space-y-2">
                <Label htmlFor="csv-delimiter">CSV 分隔符</Label>
                <Select
                  value={csvDelimiter}
                  onValueChange={(value) =>
                    setCsvDelimiter(value as ',' | ';' | '\t')
                  }
                >
                  <SelectTrigger id="csv-delimiter">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value=",">逗号 (,)</SelectItem>
                    <SelectItem value=";">分号 (;)</SelectItem>
                    <SelectItem value="\t">制表符 (Tab)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}

            {format === 'markdown' && (
              <div className="space-y-2">
                <Label htmlFor="heading-level">标题级别</Label>
                <Select
                  value={markdownHeadingLevel.toString()}
                  onValueChange={(value) =>
                    setMarkdownHeadingLevel(parseInt(value) as 1 | 2 | 3)
                  }
                >
                  <SelectTrigger id="heading-level">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="1">一级标题 (#)</SelectItem>
                    <SelectItem value="2">二级标题 (##)</SelectItem>
                    <SelectItem value="3">三级标题 (###)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            )}
          </div>

          {/* 数据摘要 */}
          <div className="p-3 bg-muted/50 rounded-md">
            <p className="text-sm text-muted-foreground">
              将导出 <span className="font-medium text-foreground">{data.messages.length}</span>{' '}
              条消息
              {data.stats?.totalTokens && (
                <>
                  ，<span className="font-medium text-foreground">{data.stats.totalTokens}</span>{' '}
                  Tokens
                </>
              )}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={handleCancel} disabled={isExporting}>
            取消
          </Button>
          <Button onClick={handleExport} disabled={isExporting}>
            {isExporting ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                导出中...
              </>
            ) : (
              <>
                <Download className="h-4 w-4 mr-2" />
                导出
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
