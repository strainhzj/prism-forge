/**
 * 前端性能基准测试
 *
 * 测试核心场景的性能指标：
 * - 会话列表渲染（1000+项）<500ms
 * - 消息树展开/折叠 <100ms
 * - 代码 Diff 渲染（1000+行）<1s
 */

import { describe, it, expect, afterEach } from 'vitest';
import { render, cleanup } from '@testing-library/react';
// import { SessionList } from '@/components/SessionList'; // 已删除
import { MessageTree } from '@/components/MessageTree';
import { DiffViewer } from '@/components/DiffViewer';
import type { Session } from '@/stores/useSessionStore';

describe('前端性能基准测试', () => {
  afterEach(() => {
    cleanup();
  });

  /**
   * 性能测试辅助函数
   */
  function measurePerformance<T>(
    fn: () => T
  ): { result: T; duration: number } {
    const start = performance.now();
    const result = fn();
    const end = performance.now();
    const duration = end - start;
    return { result, duration };
  }

  /**
   * 生成模拟会话数据
   */
  function generateMockSessions(count: number): Session[] {
    return Array.from({ length: count }, (_, i) => ({
      sessionId: `session-${i}`,
      projectName: `Project ${i}`,
      projectPath: `/path/to/project-${i}`,
      filePath: `/path/to/session-${i}.json`,
      createdAt: new Date(Date.now() - i * 1000000).toISOString(),
      updatedAt: new Date(Date.now() - i * 500000).toISOString(),
      rating: i % 6,
      tags: i % 3 === 0 ? '["test", "performance"]' : '[]',
      isArchived: i % 10 === 0,
      isActive: i === 0,
    }));
  }

  /**
   * 生成模拟代码数据
   */
  function generateMockCode(lines: number): string {
    const line = 'function test() { console.log("performance test"); }';
    return Array.from({ length: lines }, () => line).join('\n');
  }

  // SessionList 组件已删除，以下测试暂时禁用
  describe.skip('会话列表渲染性能', () => {
    it('应该在 500ms 内渲染 1000 个会话卡片', () => {
      generateMockSessions(1000);

      const { duration } = measurePerformance(() => {
        // render(<SessionList />);
      });

      console.log(`[性能] 1000 个会话渲染耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(500);
    });

    it('应该在 200ms 内渲染 100 个会话卡片', () => {
      generateMockSessions(100);

      const { duration } = measurePerformance(() => {
        // render(<SessionList />);
      });

      console.log(`[性能] 100 个会话渲染耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(200);
    });
  });

  describe('消息树性能', () => {
    it('应该在 100ms 内展开/折叠消息节点', async () => {
      const filePath = '/mock/path/to/session.json';

      const { duration } = measurePerformance(() => {
        render(<MessageTree filePath={filePath} lazy={false} />);
      });

      console.log(`[性能] 消息树展开/折叠耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(100);
    });

    it('应该在 200ms 内处理深度嵌套的消息树', () => {
      const deepFilePath = '/mock/path/to/deep-session.json';

      const { duration } = measurePerformance(() => {
        render(<MessageTree filePath={deepFilePath} lazy={false} />);
      });

      console.log(`[性能] 深度嵌套消息树处理耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(200);
    });
  });

  describe('代码 Diff 渲染性能', () => {
    it('应该在 1s 内渲染 1000 行代码 Diff', () => {
      const oldCode = generateMockCode(1000);
      const newCode = generateMockCode(1000);

      const { duration } = measurePerformance(() => {
        render(
          <DiffViewer
            oldValue={oldCode}
            newValue={newCode}
            changeInfo={{
              file_path: '/test/file.ts',
              old_text: oldCode,
              new_text: newCode,
              start_line: 1,
              end_line: 1000,
              change_type: 'update',
            }}
          />
        );
      });

      console.log(`[性能] 1000 行代码 Diff 渲染耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(1000);
    });

    it('应该在 500ms 内渲染 500 行代码 Diff', () => {
      const oldCode = generateMockCode(500);
      const newCode = generateMockCode(500);

      const { duration } = measurePerformance(() => {
        render(
          <DiffViewer
            oldValue={oldCode}
            newValue={newCode}
            changeInfo={{
              file_path: '/test/file.ts',
              old_text: oldCode,
              new_text: newCode,
              start_line: 1,
              end_line: 500,
              change_type: 'update',
            }}
          />
        );
      });

      console.log(`[性能] 500 行代码 Diff 渲染耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(500);
    });

    it('应该在 200ms 内切换 Diff 视图模式', () => {
      const oldCode = generateMockCode(100);
      const newCode = generateMockCode(100);

      const { rerender } = render(
        <DiffViewer
          oldValue={oldCode}
          newValue={newCode}
          changeInfo={{
            file_path: '/test/file.ts',
            old_text: oldCode,
            new_text: newCode,
            start_line: 1,
            end_line: 100,
            change_type: 'update',
          }}
          defaultViewMode="split"
        />
      );

      const { duration } = measurePerformance(() => {
        rerender(
          <DiffViewer
            oldValue={oldCode}
            newValue={newCode}
            changeInfo={{
              file_path: '/test/file.ts',
              old_text: oldCode,
              new_text: newCode,
              start_line: 1,
              end_line: 100,
              change_type: 'update',
            }}
            defaultViewMode="unified"
          />
        );
      });

      console.log(`[性能] Diff 视图模式切换耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(200);
    });
  });

  // SessionList 组件已删除，以下测试暂时禁用
  describe.skip('内存泄漏检测', () => {
    it('应该在多次渲染后释放内存', () => {
      const initialMemory = (performance as any).memory?.usedJSHeapSize || 0;

      // 执行多次渲染
      for (let i = 0; i < 10; i++) {
        // render(<SessionList />);
        cleanup();
      }

      // 强制垃圾回收（如果可用）
      if (typeof globalThis !== 'undefined' && (globalThis as any).gc) {
        (globalThis as any).gc();
      }

      const finalMemory = (performance as any).memory?.usedJSHeapSize || 0;
      const memoryIncrease = finalMemory - initialMemory;

      console.log(`[内存] 内存增长: ${(memoryIncrease / 1024 / 1024).toFixed(2)}MB`);

      // 内存增长应该小于 10MB
      expect(memoryIncrease).toBeLessThan(10 * 1024 * 1024);
    });
  });

  // SessionList 组件已删除，以下测试暂时禁用
  describe.skip('交互响应性能', () => {
    it('应该在 50ms 内响应点击事件', () => {
      generateMockSessions(10);

      // const { container } = render(<SessionList />);

      const { duration } = measurePerformance(() => {
        // const button = container.querySelector('button');
        // if (button) {
        //   button.click();
        // }
      });

      console.log(`[交互] 点击事件响应耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(50);
    });

    it('应该在 100ms 内更新过滤结果', () => {
      // const { rerender } = render(<SessionList />);

      const { duration } = measurePerformance(() => {
        // 模拟搜索输入
        // rerender(<SessionList />);
      });

      console.log(`[交互] 过滤更新耗时: ${duration.toFixed(2)}ms`);
      expect(duration).toBeLessThan(100);
    });
  });
});
