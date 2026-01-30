import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import { Home, Trash2 } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { ScenarioTemplateList } from '@/components/prompt-versions';

/**
 * 提示词模板管理页面
 *
 * 显示场景级别的模板（如"会话分析"），支持编辑
 */
export default function PromptsPage() {
  const { t } = useTranslation('prompts');
  const navigate = useNavigate();
  const [cleaning, setCleaning] = useState(false);

  // 清理历史数据
  const handleCleanup = async () => {
    if (!confirm('确定要清理所有非"会话分析"的模板吗？此操作不可撤销。')) {
      return;
    }

    setCleaning(true);
    try {
      const deleted = await invoke<number>('cmd_cleanup_legacy_templates');
      alert(`成功清理 ${deleted} 个历史模板`);
      // 刷新页面
      window.location.reload();
    } catch (error) {
      console.error('清理失败:', error);
      alert('清理失败：' + error);
    } finally {
      setCleaning(false);
    }
  };

  return (
    <div className="container mx-auto px-4 py-8">
      {/* 页面标题 + 返回按钮 */}
      <div className="mb-6 flex items-center gap-4">
        <button
          className="back-btn"
          onClick={() => navigate('/')}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            padding: '8px 16px',
            background: 'var(--color-bg-card)',
            color: 'var(--color-accent-warm)',
            border: '1px solid var(--color-accent-warm)',
            borderRadius: '8px',
            fontSize: '14px',
            fontWeight: '500',
            cursor: 'pointer',
            transition: 'all 0.2s',
            flexShrink: 0,
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.background = 'color-mix(in srgb, var(--color-accent-warm) 15%, var(--color-bg-card))';
            e.currentTarget.style.transform = 'translateY(-1px)';
            e.currentTarget.style.boxShadow = '0 0 10px var(--color-accent-warm-shadow)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.background = 'var(--color-bg-card)';
            e.currentTarget.style.transform = 'translateY(0)';
            e.currentTarget.style.boxShadow = 'none';
          }}
        >
          <Home size={16} />
          <span>{t('backToHome')}</span>
        </button>
        <div className="flex-1">
          <h1
            className="text-3xl font-bold"
            style={{ color: 'var(--color-text-primary)' }}
          >
            {t('title')}
          </h1>
          <p
            className="mt-2"
            style={{ color: 'var(--color-text-secondary)' }}
          >
            管理提示词模板，编辑 Meta-Prompt 组件
          </p>
        </div>
      </div>

      {/* 场景模板列表 */}
      <div className="rounded-lg border p-6" style={{
        backgroundColor: 'var(--color-bg-card)',
        borderColor: 'var(--color-border-light)',
      }}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold" style={{ color: 'var(--color-text-primary)' }}>
            提示词模板
          </h2>
          <button
            onClick={handleCleanup}
            disabled={cleaning}
            className="flex items-center gap-2 px-3 py-1.5 rounded-md text-sm transition-colors disabled:opacity-50"
            style={{
              backgroundColor: 'var(--color-bg-primary)',
              color: 'var(--color-accent-red)',
              border: '1px solid var(--color-accent-red)',
            }}
            title="清理所有非会话分析的历史模板"
          >
            <Trash2 className="w-4 h-4" />
            <span>{cleaning ? '清理中...' : '清理历史数据'}</span>
          </button>
        </div>
        {/* 不传递 onEditTemplate，让组件自己管理编辑状态 */}
        <ScenarioTemplateList />
      </div>
    </div>
  );
}
