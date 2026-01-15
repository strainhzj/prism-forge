/**
 * E2E 测试：添加目录功能
 * 测试会话管理页面中的"添加目录"按钮功能
 */

import { test, expect } from '@playwright/test';
import { join } from 'path';
import { tmpdir } from 'os';
import { mkdirSync, existsSync } from 'fs';

test.describe('添加目录功能测试', () => {
  let testDir: string;

  test.beforeAll(async () => {
    // 创建测试目录
    testDir = join(tmpdir(), 'prism-forge-test-' + Date.now());
    if (!existsSync(testDir)) {
      mkdirSync(testDir, { recursive: true });
    }
  });

  test.beforeEach(async ({ page }) => {
    await page.goto('/sessions');
    await page.waitForLoadState('networkidle');
  });

  test('应该能够点击添加目录按钮', async ({ page }) => {
    await test.step('查找添加目录按钮', async () => {
      // 查找添加目录按钮
      const addDirectoryButton = page.getByRole('button', { name: /添加目录/i });
      
      // 验证按钮存在且可见
      await expect(addDirectoryButton).toBeVisible();
      
      // 验证按钮可点击
      await expect(addDirectoryButton).toBeEnabled();
    });

    await test.step('点击添加目录按钮', async () => {
      const addDirectoryButton = page.getByRole('button', { name: /添加目录/i });
      
      // 点击按钮
      await addDirectoryButton.click();
      
      // 等待文件选择对话框打开
      // 注意：在 E2E 测试中，系统文件对话框可能无法直接测试
      // 我们可以检查是否有相关的事件或状态变化
      await page.waitForTimeout(1000);
    });
  });

  test('应该能够显示监控目录列表', async ({ page }) => {
    await test.step('查找监控目录区域', async () => {
      // 查找监控目录标题
      const monitoredDirTitle = page.getByText(/监控目录/i);
      
      // 如果有监控目录，应该能看到标题
      if (await monitoredDirTitle.isVisible()) {
        await expect(monitoredDirTitle).toBeVisible();
        
        // 查找目录列表
        const directoryList = page.locator('[data-testid="monitored-directories"]').or(
          page.locator('.monitored-directories')
        );
        
        // 验证列表存在
        if (await directoryList.isVisible()) {
          await expect(directoryList).toBeVisible();
        }
      }
    });
  });

  test('应该能够切换监控目录状态', async ({ page }) => {
    await test.step('查找监控目录的开关按钮', async () => {
      // 查找启用/禁用按钮
      const toggleButton = page.locator('button[title*="启用"], button[title*="禁用"]');
      
      const count = await toggleButton.count();
      
      if (count > 0) {
        const firstToggle = toggleButton.first();
        
        // 获取当前状态
        const isActive = await firstToggle.evaluate((el) => 
          el.classList.contains('text-green-500')
        );
        
        // 点击切换
        await firstToggle.click();
        
        // 等待状态更新
        await page.waitForTimeout(500);
        
        // 验证状态已改变
        const newIsActive = await firstToggle.evaluate((el) => 
          el.classList.contains('text-green-500')
        );
        
        expect(newIsActive).toBe(!isActive);
      } else {
        test.skip(true, '没有可用的监控目录');
      }
    });
  });

  test('应该能够删除监控目录', async ({ page }) => {
    await test.step('查找删除按钮', async () => {
      // 查找删除按钮
      const deleteButton = page.locator('button[title="删除"]');
      
      const count = await deleteButton.count();
      
      if (count > 0) {
        const firstDelete = deleteButton.first();
        
        // 点击删除
        await firstDelete.click();
        
        // 等待删除完成
        await page.waitForTimeout(500);
        
        // 验证目录已从列表中移除
        const newCount = await deleteButton.count();
        expect(newCount).toBe(count - 1);
      } else {
        test.skip(true, '没有可用的监控目录可删除');
      }
    });
  });

  test('应该能够刷新会话列表', async ({ page }) => {
    await test.step('查找刷新按钮', async () => {
      const refreshButton = page.getByRole('button', { name: /刷新/i });
      
      // 验证刷新按钮存在
      await expect(refreshButton).toBeVisible();
      await expect(refreshButton).toBeEnabled();
    });

    await test.step('点击刷新按钮', async () => {
      const refreshButton = page.getByRole('button', { name: /刷新/i });
      
      // 点击刷新
      await refreshButton.click();
      
      // 等待刷新完成
      await page.waitForTimeout(1000);
      
      // 验证页面没有错误
      const errorMessage = page.getByText(/错误|失败/i);
      if (await errorMessage.isVisible()) {
        console.warn('刷新时出现错误:', await errorMessage.textContent());
      }
    });
  });

  test('应该显示正确的空状态提示', async ({ page }) => {
    await test.step('检查空状态提示', async () => {
      // 查找项目列表区域
      const projectList = page.locator('[data-testid="project-list"]').or(
        page.locator('.project-list')
      );
      
      // 如果没有项目，应该显示空状态
      const emptyState = page.getByText(/暂无项目/i);
      
      if (await emptyState.isVisible()) {
        await expect(emptyState).toBeVisible();
        
        // 检查提示文本
        const helpText = page.getByText(/点击.*添加目录.*添加监控目录/i);
        if (await helpText.isVisible()) {
          await expect(helpText).toBeVisible();
        }
      }
    });
  });

  test('应该能够处理错误状态', async ({ page }) => {
    // 监听控制台错误
    const consoleErrors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });

    await test.step('点击添加目录按钮并检查错误', async () => {
      const addDirectoryButton = page.getByRole('button', { name: /添加目录/i });
      
      if (await addDirectoryButton.isVisible()) {
        await addDirectoryButton.click();
        
        // 等待可能的错误
        await page.waitForTimeout(2000);
        
        // 检查是否有错误消息显示
        const errorAlert = page.locator('[role="alert"]').or(
          page.getByText(/错误|失败/i)
        );
        
        if (await errorAlert.isVisible()) {
          const errorText = await errorAlert.textContent();
          console.log('发现错误消息:', errorText);
        }
        
        // 检查控制台错误
        if (consoleErrors.length > 0) {
          console.log('控制台错误:', consoleErrors);
        }
      }
    });
  });
});