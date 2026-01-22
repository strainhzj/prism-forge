import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Settings, RotateCcw } from "lucide-react";
import { cn } from "@/lib/utils";
import type { OptimizerConfig } from "@/types/generated";

export function OptimizerSettings() {
  const [config, setConfig] = useState<OptimizerConfig | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error', text: string } | null>(null);

  // 加载配置
  const loadConfig = async () => {
    setLoading(true);
    try {
      const configJson = await invoke<string>("get_optimizer_config");
      const parsedConfig = JSON.parse(configJson) as OptimizerConfig;
      setConfig(parsedConfig);
      setMessage({ type: 'success', text: '配置加载成功' });
    } catch (error) {
      console.error('加载配置失败:', error);
      setMessage({ type: 'error', text: `加载失败: ${error}` });
    } finally {
      setLoading(false);
    }
  };

  // 重新加载配置
  const reloadConfig = async () => {
    setSaving(true);
    try {
      await invoke<string>("reload_optimizer_config");
      await loadConfig();
      setMessage({ type: 'success', text: '配置已重新加载' });
    } catch (error) {
      console.error('重新加载配置失败:', error);
      setMessage({ type: 'error', text: `重新加载失败: ${error}` });
    } finally {
      setSaving(false);
    }
  };

  useEffect(() => {
    loadConfig();
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <RefreshCw className="h-6 w-6 animate-spin" />
        <span className="ml-2">加载配置中...</span>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="p-8 text-center">
        <p className="text-red-500">无法加载配置</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      {/* 标题栏 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Settings className="h-5 w-5" />
          <h2 className="text-xl font-semibold">优化器配置</h2>
        </div>
        <div className="flex gap-2">
          <button
            onClick={reloadConfig}
            disabled={saving}
            className={cn(
              "flex items-center gap-2 px-4 py-2 rounded-lg transition-colors",
              "bg-blue-500 hover:bg-blue-600 text-white",
              "disabled:opacity-50 disabled:cursor-not-allowed"
            )}
          >
            <RotateCcw className="h-4 w-4" />
            重新加载
          </button>
        </div>
      </div>

      {/* 消息提示 */}
      {message && (
        <div
          className={cn(
            "p-3 rounded-lg",
            message.type === 'success'
              ? "bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300"
              : "bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300"
          )}
        >
          {message.text}
        </div>
      )}

      {/* Meta-Prompt 配置 - 提示用户直接编辑配置文件 */}
      <div className="space-y-2">
        <h3 className="text-lg font-medium">Meta-Prompt 模板</h3>
        <p className="text-sm text-gray-600 dark:text-gray-400">
          模板支持多语言（中文 template_zh 和英文 template_en）
        </p>
        <div className="p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 text-sm">
          <p>提示词模板已在配置文件中支持中英文双语切换。</p>
          <p className="mt-1">如需修改模板内容，请直接编辑文件：</p>
          <code className="block mt-1 p-1 bg-white dark:bg-gray-800 rounded">src-tauri/optimizer_config.toml</code>
        </div>
      </div>

      {/* LLM 参数配置 */}
      <div className="space-y-2">
        <h3 className="text-lg font-medium">LLM 调用参数</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium mb-1">Temperature</label>
            <input
              type="number"
              step="0.1"
              min="0"
              max="1"
              value={config.llm_params.temperature}
              onChange={(e) => setConfig({
                ...config,
                llm_params: { ...config.llm_params, temperature: parseFloat(e.target.value) }
              })}
              className="w-full px-3 py-2 rounded-lg border"
              style={{
                backgroundColor: 'var(--color-bg-card)',
                borderColor: 'var(--color-border-light)'
              }}
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">Max Tokens</label>
            <input
              type="number"
              min="100"
              max="4000"
              value={config.llm_params.max_tokens}
              onChange={(e) => setConfig({
                ...config,
                llm_params: { ...config.llm_params, max_tokens: parseInt(e.target.value) }
              })}
              className="w-full px-3 py-2 rounded-lg border"
              style={{
                backgroundColor: 'var(--color-bg-card)',
                borderColor: 'var(--color-border-light)'
              }}
            />
          </div>
        </div>
      </div>

      {/* 会话上下文配置 */}
      <div className="space-y-2">
        <h3 className="text-lg font-medium">会话上下文配置</h3>
        <div className="space-y-3">
          <div>
            <label className="block text-sm font-medium mb-1">最大摘要长度</label>
            <input
              type="number"
              min="50"
              max="500"
              value={config.session_context.max_summary_length}
              onChange={(e) => setConfig({
                ...config,
                session_context: { ...config.session_context, max_summary_length: parseInt(e.target.value) }
              })}
              className="w-full px-3 py-2 rounded-lg border"
              style={{
                backgroundColor: 'var(--color-bg-card)',
                borderColor: 'var(--color-border-light)'
              }}
            />
          </div>
          <div className="flex gap-4">
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.session_context.include_rating}
                onChange={(e) => setConfig({
                  ...config,
                  session_context: { ...config.session_context, include_rating: e.target.checked }
                })}
                className="rounded"
              />
              <span className="text-sm">包含评分</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.session_context.include_project}
                onChange={(e) => setConfig({
                  ...config,
                  session_context: { ...config.session_context, include_project: e.target.checked }
                })}
                className="rounded"
              />
              <span className="text-sm">包含项目名</span>
            </label>
          </div>
        </div>
      </div>

      {/* 高级设置 */}
      <div className="space-y-2">
        <h3 className="text-lg font-medium">高级设置</h3>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium mb-1">并行处理数</label>
            <input
              type="number"
              min="1"
              max="10"
              value={config.advanced.parallel_processing}
              onChange={(e) => setConfig({
                ...config,
                advanced: { ...config.advanced, parallel_processing: parseInt(e.target.value) }
              })}
              className="w-full px-3 py-2 rounded-lg border"
              style={{
                backgroundColor: 'var(--color-bg-card)',
                borderColor: 'var(--color-border-light)'
              }}
            />
          </div>
          <div>
            <label className="block text-sm font-medium mb-1">超时时间 (秒)</label>
            <input
              type="number"
              min="5"
              max="120"
              value={config.advanced.timeout}
              onChange={(e) => setConfig({
                ...config,
                advanced: { ...config.advanced, timeout: parseInt(e.target.value) }
              })}
              className="w-full px-3 py-2 rounded-lg border"
              style={{
                backgroundColor: 'var(--color-bg-card)',
                borderColor: 'var(--color-border-light)'
              }}
            />
          </div>
        </div>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={config.advanced.debug}
            onChange={(e) => setConfig({
              ...config,
              advanced: { ...config.advanced, debug: e.target.checked }
            })}
            className="rounded"
          />
          <span className="text-sm">启用调试模式</span>
        </label>
      </div>

      {/* 说明 */}
      <div className="mt-6 p-4 rounded-lg bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800">
        <h4 className="font-medium mb-2">配置说明</h4>
        <ul className="text-sm space-y-1 list-disc list-inside">
          <li>修改配置后需要点击"重新加载"按钮才能生效</li>
          <li>配置文件位于 <code>src-tauri/optimizer_config.toml</code></li>
          <li>详细配置说明请参考 <code>OPTIMIZER_CONFIG_GUIDE.md</code></li>
        </ul>
      </div>
    </div>
  );
}
