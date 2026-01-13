/**
 * 诊断页面 - 用于检查 UI 更新问题
 */

import { useEffect } from 'react';
import React from 'react';

export function DiagnosticPage() {
  useEffect(() => {
    console.log('=== 诊断信息 ===');
    console.log('当前页面:', window.location.pathname);
    console.log('React 版本:', React.version);
    console.log('所有已加载模块:', performance.getEntriesByType('resource').map(r => r.name));
  }, []);

  return (
    <div style={{ padding: '20px', fontFamily: 'monospace' }}>
      <h1>🔍 UI 诊断页面</h1>

      <div style={{ marginBottom: '20px', padding: '10px', background: '#f0f0f0', borderRadius: '5px' }}>
        <h2>✅ 新组件测试</h2>
        <p>如果你能看到这个页面，说明 React 正在正常工作。</p>
        <p><strong>下一步：</strong></p>
        <ol>
          <li>打开浏览器开发者工具（F12）</li>
          <li>查看 Console 标签</li>
          <li>确认有 "=== 诊断信息 ===" 输出</li>
        </ol>
      </div>

      <div style={{ marginBottom: '20px', padding: '10px', background: '#e3f2fd', borderRadius: '5px' }}>
        <h2>📋 路由测试</h2>
        <p>请尝试访问以下 URL：</p>
        <ul>
          <li><a href="/sessions">会话列表</a></li>
          <li><a href="/settings">设置页面</a></li>
        </ul>
      </div>

      <div style={{ marginBottom: '20px', padding: '10px', background: '#fff3e0', borderRadius: '5px' }}>
        <h2>🔧 检查清单</h2>
        <ul>
          <li>✅ 浏览器控制台是否有错误？</li>
          <li>✅ Network 标签是否加载了 SessionDetailPageV2.js？</li>
          <li>✅ 是否看到了 [SessionDetailPageV2] 开头的日志？</li>
        </ul>
      </div>

      <div style={{ padding: '10px', background: '#f1f8e9', borderRadius: '5px' }}>
        <h2>💡 如果新 UI 没有显示</h2>
        <p>请执行以下操作：</p>
        <ol>
          <li><strong>硬刷新浏览器：</strong> Ctrl + Shift + R（Windows）或 Cmd + Shift + R（Mac）</li>
          <li><strong>清除缓存：</strong> F12 → Application → Clear storage → Clear site data</li>
          <li><strong>隐私模式：</strong> 在无痕/隐私模式下打开浏览器</li>
          <li><strong>查看控制台：</strong> 检查是否有 JavaScript 错误</li>
        </ol>
      </div>
    </div>
  );
}
