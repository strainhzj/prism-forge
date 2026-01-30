import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { PromptVersion } from '@/types/generated';
import { X, AlertCircle } from 'lucide-react';

interface PromptEditDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  templateName: string;
  currentVersion: PromptVersion | null;
  onSaveSuccess?: () => void;
}

interface ComponentData {
  zh: {
    meta_prompt: {
      content: string;
      last_modified: string | null;
    };
    input_template: {
      content: string;
      last_modified: string | null;
    };
    output_template: {
      content: string;
      last_modified: string | null;
    };
  };
  en: {
    meta_prompt: {
      content: string;
      last_modified: string | null;
    };
    input_template: {
      content: string;
      last_modified: string | null;
    };
    output_template: {
      content: string;
      last_modified: string | null;
    };
  };
}

type Language = 'zh' | 'en';

/**
 * 提示词编辑抽屉组件
 *
 * 功能：
 * - 显示配置文件更新警告
 * - 支持中英文标签页切换
 * - 编辑 Meta-Prompt 组件
 * - 保存时创建新版本
 */
export function PromptEditDrawer({
  isOpen,
  onClose,
  templateName,
  currentVersion,
  onSaveSuccess,
}: PromptEditDrawerProps) {
  const [currentLanguage, setCurrentLanguage] = useState<Language>('zh');
  const [componentData, setComponentData] = useState<ComponentData | null>(null);
  const [editedContent, setEditedContent] = useState('');
  const [originalContent, setOriginalContent] = useState('');
  const [configUpdated, setConfigUpdated] = useState(false);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [showWarning, setShowWarning] = useState(false);

  // 加载组件数据
  useEffect(() => {
    if (!isOpen || !templateName) return;

    const loadComponentData = async () => {
      setLoading(true);
      try {
        // 获取组件化数据
        const data = await invoke<ComponentData>('cmd_get_prompt_components', {
          templateName,
        });

        setComponentData(data);

        // 初始化编辑内容
        const initialContent = data[currentLanguage].meta_prompt.content;
        setEditedContent(initialContent);
        setOriginalContent(initialContent);

        // 检查配置文件是否已更新
        const updated = await invoke<boolean>('cmd_check_config_updated', {
          templateName,
        });
        setConfigUpdated(updated);
        setShowWarning(updated);
      } catch (error) {
        console.error('加载组件数据失败:', error);
      } finally {
        setLoading(false);
      }
    };

    loadComponentData();
  }, [isOpen, templateName]);

  // 切换语言时临时保存当前编辑内容
  const handleLanguageSwitch = (newLanguage: Language) => {
    if (!componentData) return;

    // 临时保存当前编辑内容到状态
    setComponentData((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        [currentLanguage]: {
          ...prev[currentLanguage],
          meta_prompt: {
            ...prev[currentLanguage].meta_prompt,
            content: editedContent,
          },
        },
      };
    });

    // 切换到新语言
    const newContent = componentData[newLanguage].meta_prompt.content;
    setEditedContent(newContent);
    setOriginalContent(newContent);
    setCurrentLanguage(newLanguage);
  };

  // 检测用户开始编辑
  useEffect(() => {
    if (componentData && editedContent !== originalContent) {
      // 检查另一个语言是否修改
      const otherLanguage: Language = currentLanguage === 'zh' ? 'en' : 'zh';
      const otherOriginal = componentData[otherLanguage].meta_prompt.last_modified;

      // 如果另一个语言没有修改时间，说明是初始版本（未修改）
      const isOtherUnmodified = !otherOriginal;

      if (isOtherUnmodified && !showWarning) {
        setShowWarning(true);
      }
    }
  }, [editedContent, originalContent, componentData, currentLanguage, showWarning]);

  // 保存修改
  const handleSave = async () => {
    if (!componentData || saving) return;

    setSaving(true);
    try {
      // 构建更新后的组件数据
      const updatedData: ComponentData = {
        ...componentData,
        [currentLanguage]: {
          ...componentData[currentLanguage],
          meta_prompt: {
            content: editedContent,
            last_modified: new Date().toISOString(),
          },
        },
      };

      // 确定哪些语言被更新了
      const updatedLanguages: Language[] = [currentLanguage];

      // 调用后端命令更新组件
      await invoke<PromptVersion>('cmd_update_prompt_components', {
        templateName,
        componentsData: JSON.stringify(updatedData),
        updatedLanguages,
      });

      // 保存成功回调
      onSaveSuccess?.();
      onClose();
    } catch (error) {
      console.error('保存失败:', error);
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex"
      style={{ backgroundColor: 'rgba(0, 0, 0, 0.5)' }}
      onClick={onClose}
    >
      <div
        className="ml-auto h-full w-full max-w-4xl overflow-hidden flex flex-col"
        style={{ backgroundColor: 'var(--color-bg-card)' }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
          <div className="flex-1">
            <h2 className="text-xl font-semibold" style={{ color: 'var(--color-text-primary)' }}>
              {templateName === 'session_analysis' ? '编辑会话分析提示词' : `编辑 ${templateName}`}
              {currentVersion && ` - v${currentVersion.versionNumber}`}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            style={{ color: 'var(--color-text-secondary)' }}
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Warning Banner */}
        {showWarning && (
          <div className="mx-6 mt-4 p-3 rounded-md flex items-start gap-2" style={{
            backgroundColor: 'rgba(245, 158, 11, 0.1)',
            border: '1px solid rgba(245, 158, 11, 0.3)',
          }}>
            <AlertCircle className="w-5 h-5 mt-0.5 flex-shrink-0" style={{ color: 'var(--color-accent-warm)' }} />
            <div className="flex-1 text-sm" style={{ color: 'var(--color-text-primary)' }}>
              {configUpdated ? (
                <>
                  <strong>配置文件已更新：</strong>只读组件已同步，请检查 Meta-Prompt 是否需要调整。
                </>
              ) : (
                <>
                  <strong>提示：</strong>
                  {currentLanguage === 'zh'
                    ? '英文版本尚未修改，内容仍为初始版本。'
                    : '中文版本尚未修改，内容仍为初始版本。'}
                </>
              )}
            </div>
            <button
              onClick={() => setShowWarning(false)}
              className="text-sm underline hover:opacity-80"
              style={{ color: 'var(--color-text-secondary)' }}
            >
              关闭
            </button>
          </div>
        )}

        {/* Language Tabs */}
        <div className="px-6 pt-4">
          <div className="flex gap-2 border-b" style={{ borderColor: 'var(--color-border-light)' }}>
            <button
              onClick={() => handleLanguageSwitch('zh')}
              className={`px-4 py-2 text-sm font-medium transition-colors ${
                currentLanguage === 'zh'
                  ? 'border-b-2'
                  : 'hover:opacity-80'
              }`}
              style={{
                color: currentLanguage === 'zh' ? 'var(--color-accent-blue)' : 'var(--color-text-secondary)',
                borderColor: currentLanguage === 'zh' ? 'var(--color-accent-blue)' : 'transparent',
              }}
            >
              中文
            </button>
            <button
              onClick={() => handleLanguageSwitch('en')}
              className={`px-4 py-2 text-sm font-medium transition-colors ${
                currentLanguage === 'en'
                  ? 'border-b-2'
                  : 'hover:opacity-80'
              }`}
              style={{
                color: currentLanguage === 'en' ? 'var(--color-accent-blue)' : 'var(--color-text-secondary)',
                borderColor: currentLanguage === 'en' ? 'var(--color-accent-blue)' : 'transparent',
              }}
            >
              English
            </button>
          </div>
        </div>

        {/* Editor */}
        <div className="flex-1 p-6 overflow-auto">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center" style={{ color: 'var(--color-text-secondary)' }}>
                <div className="animate-spin rounded-full h-8 w-8 border-b-2 mx-auto mb-2" style={{ borderColor: 'var(--color-accent-blue)' }}></div>
                <p>加载中...</p>
              </div>
            </div>
          ) : (
            <div className="h-full flex flex-col">
              <label className="block text-sm font-medium mb-2" style={{ color: 'var(--color-text-primary)' }}>
                Meta-Prompt
              </label>
              <textarea
                value={editedContent}
                onChange={(e) => setEditedContent(e.target.value)}
                className="flex-1 w-full p-4 rounded-md border font-mono text-sm resize-none focus:outline-none focus:ring-2"
                style={{
                  backgroundColor: 'var(--color-bg-primary)',
                  borderColor: 'var(--color-border-light)',
                  color: 'var(--color-text-primary)',
                }}
                placeholder="输入 Meta-Prompt 内容..."
              />
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t" style={{ borderColor: 'var(--color-border-light)' }}>
          <button
            onClick={onClose}
            disabled={saving}
            className="px-4 py-2 rounded-md text-sm font-medium transition-colors"
            style={{
              backgroundColor: 'var(--color-bg-primary)',
              color: 'var(--color-text-primary)',
              opacity: saving ? 0.5 : 1,
            }}
          >
            取消
          </button>
          <button
            onClick={handleSave}
            disabled={saving || editedContent === originalContent}
            className="px-4 py-2 rounded-md text-sm font-medium text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            style={{
              backgroundColor: 'var(--color-accent-blue)',
            }}
          >
            {saving ? '保存中...' : '保存'}
          </button>
        </div>
      </div>
    </div>
  );
}
