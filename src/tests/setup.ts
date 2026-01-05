/**
 * Vitest 测试环境设置
 */

import { expect, afterEach, vi } from 'vitest';
import { cleanup } from '@testing-library/react';
import * as matchers from '@testing-library/jest-dom/matchers';

// 扩展 Vitest 的 expect 断言
expect.extend(matchers);

// 每个测试后清理 DOM
afterEach(() => {
  cleanup();
});

// Mock Tauri API
Object.assign(globalThis as any, {
  __TAURI__: {
    tauri: {
      invoke: vi.fn(),
    },
    core: {
      path: {
        resolve: vi.fn(),
      },
    },
  },
});

// Mock window.__TAURI__
Object.defineProperty(window, '__TAURI__', {
  value: (globalThis as any).__TAURI__,
  writable: true,
});

// Mock Performance API 用于性能测试
globalThis.performance = {
  ...performance,
  now: vi.fn(() => Date.now()),
} as Performance;

// Mock performance.memory（仅在支持的浏览器中可用）
if (!(performance as any).memory) {
  Object.defineProperty(performance, 'memory', {
    value: {
      usedJSHeapSize: 10000000,
      totalJSHeapSize: 20000000,
      jsHeapSizeLimit: 1000000000,
    },
    writable: true,
  });
}
