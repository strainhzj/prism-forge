import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { PromptTemplate, PromptVersion } from '@/types/generated';
import { Edit, Eye } from 'lucide-react';
import { PromptEditDrawer } from './PromptEditDrawer';

interface ScenarioTemplateListProps {
  onEditTemplate?: (templateName: string, currentVersion: PromptVersion) => void;
}

interface ScenarioTemplate {
  template: PromptTemplate;
  activeVersion: PromptVersion | null;
}

/**
 * 场景模板列表组件
 *
 * 只显示场景级别的模板（如"会话分析"），不显示具体的版本
 */
export function ScenarioTemplateList({ onEditTemplate }: ScenarioTemplateListProps) {
  const [templates, setTemplates] = useState<ScenarioTemplate[]>([]);
  const [loading, setLoading] = useState(true);
  const [editDrawerOpen, setEditDrawerOpen] = useState(false);
  const [selectedTemplate, setSelectedTemplate] = useState<PromptTemplate | null>(null);
  const [selectedVersion, setSelectedVersion] = useState<PromptVersion | null>(null);

  // 加载场景模板
  const loadTemplates = async () => {
    setLoading(true);
    try {
      // 获取所有模板
      const allTemplates = await invoke<PromptTemplate[]>('cmd_get_prompt_templates');

      // 过滤出场景级别的模板（通过 scenario 字段判断）
      const scenarioTemplates: ScenarioTemplate[] = await Promise.all(
        allTemplates.map(async (template) => {
          // 获取激活版本
          let activeVersion: PromptVersion | null = null;
          if (template.id) {
            try {
              activeVersion = await invoke<PromptVersion>('cmd_get_active_prompt_version', {
                templateId: template.id,
              });
            } catch (error) {
              console.error(`获取模板 ${template.name} 的激活版本失败:`, error);
            }
          }
          return { template, activeVersion };
        })
      );

      setTemplates(scenarioTemplates);
    } catch (error) {
      console.error('加载模板失败:', error);
    } finally {
      setLoading(false);
    }
  };

  // 组件加载时获取模板
  useEffect(() => {
    loadTemplates();
  }, []);

  // 处理编辑
  const handleEdit = (template: PromptTemplate, version: PromptVersion | null) => {
    setSelectedTemplate(template);
    setSelectedVersion(version);
    if (onEditTemplate) {
      onEditTemplate(template.name, version!);
    } else {
      setEditDrawerOpen(true);
    }
  };

  // 保存成功后重新加载
  const handleSaveSuccess = () => {
    setEditDrawerOpen(false);
    loadTemplates();
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="text-center" style={{ color: 'var(--color-text-secondary)' }}>
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 mx-auto mb-2" style={{ borderColor: 'var(--color-accent-blue)' }}></div>
          <p>加载中...</p>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="space-y-3">
        {templates.map(({ template, activeVersion }) => (
          <div
            key={template.id}
            className="rounded-lg border p-4 hover:shadow-md transition-shadow"
            style={{
              backgroundColor: 'var(--color-bg-card)',
              borderColor: 'var(--color-border-light)',
            }}
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text-primary)' }}>
                  {getScenarioDisplayName(template.scenario)}
                </h3>
                <p className="text-sm mt-1" style={{ color: 'var(--color-text-secondary)' }}>
                  {template.description}
                </p>
                <div className="flex items-center gap-4 mt-2 text-xs" style={{ color: 'var(--color-text-secondary)' }}>
                  <span>场景: {template.scenario}</span>
                  {activeVersion && (
                    <span>版本: v{activeVersion.versionNumber}</span>
                  )}
                  {template.isSystem && (
                    <span className="px-2 py-0.5 rounded" style={{
                      backgroundColor: 'rgba(74, 158, 255, 0.1)',
                      color: 'var(--color-accent-blue)',
                    }}>
                      系统内置
                    </span>
                  )}
                </div>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={() => handleEdit(template, activeVersion)}
                  className="p-2 rounded-md transition-colors"
                  style={{
                    backgroundColor: 'var(--color-bg-primary)',
                    color: 'var(--color-accent-blue)',
                  }}
                  title="编辑"
                >
                  <Edit className="w-4 h-4" />
                </button>
                <button
                  className="p-2 rounded-md transition-colors"
                  style={{
                    backgroundColor: 'var(--color-bg-primary)',
                    color: 'var(--color-text-secondary)',
                  }}
                  title="查看版本"
                >
                  <Eye className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* 编辑抽屉 */}
      {(selectedTemplate || editDrawerOpen) && (
        <PromptEditDrawer
          isOpen={editDrawerOpen}
          onClose={() => {
            setEditDrawerOpen(false);
            setSelectedTemplate(null);
            setSelectedVersion(null);
          }}
          templateName={selectedTemplate?.name || 'session_analysis'}
          currentVersion={selectedVersion}
          onSaveSuccess={handleSaveSuccess}
        />
      )}
    </>
  );
}

// 辅助函数：根据场景代码获取显示名称
function getScenarioDisplayName(scenario: string): string {
  const displayNames: Record<string, string> = {
    'session_analysis': '会话分析',
    'code_generation': '代码生成',
    'code_review': '代码审查',
    'documentation': '文档生成',
  };
  return displayNames[scenario] || scenario;
}
