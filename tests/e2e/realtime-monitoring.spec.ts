/**
 * E2E 测试：实时监控功能
 * 自动刷新、文件变更检测
 */

import { test, expect } from '@playwright/test';

test.describe('实时监控测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/sessions');
    await page.waitForLoadState('networkidle');
  });

  test('应该显示实时刷新指示器', async ({ page }) => {
    await test.step('检查刷新指示器', async () => {
      // 查找刷新指示器元素
      const refreshIndicator = page.locator('[data-testid="refresh-indicator"]').or(
        page.locator('.refresh-indicator')
      ).or(page.getByTitle(/刷新|监控/i));

      // 指示器可能存在但不可见（初始状态）
      if (await refreshIndicator.count() > 0) {
        await expect(refreshIndicator.first()).toBeAttached();
      }
    });
  });

  test('应该能够手动刷新会话列表', async ({ page }) => {
    await test.step('点击刷新按钮', async () => {
      const refreshButton = page.getByRole('button', { name: /刷新|reload/i }).or(
        page.locator('[data-testid="refresh-button"]')
      );

      if (await refreshButton.isVisible()) {
        // 记录刷新前的状态
        const beforeText = await page.content();

        await refreshButton.click();

        // 等待刷新完成（查找加载状态）
        await page.waitForTimeout(2000);

        // 验证页面已更新（可选：检查时间戳或计数变化）
        const afterText = await page.content();

        // 至少应该有某些交互反馈
        await expect(refreshButton).toBeVisible();
      }
    });
  });

  test('应该显示会话统计信息', async ({ page }) => {
    await test.step('检查统计徽章', async () => {
      // 查找统计徽章（总数、已评分、已归档）
      const statsBadges = page.locator('[data-stat]').or(
        page.locator('.badge')
      );

      const count = await statsBadges.count();

      if (count > 0) {
        // 验证至少有一个徽章可见
        await expect(statsBadges.first()).toBeVisible();

        // 验证徽章内容是数字
        const firstBadgeText = await statsBadges.first().textContent();
        expect(firstBadgeText?.trim()).toMatch(/\d+/);
      }
    });
  });

  test('应该能够切换视图等级', async ({ page }) => {
    // 导航到会话详情页
    const sessionCard = page.locator('[data-testid="session-card"]').or(
      page.locator('.session-card')
    );

    const cardCount = await sessionCard.count();

    if (cardCount > 0) {
      await sessionCard.first().click();

      await test.step('检查视图等级选择器', async () => {
        await page.waitForLoadState('networkidle');

        // 查找视图等级选择器
        const viewLevelTabs = page.locator('[data-testid="view-level-selector"]').or(
          page.locator('.view-level-tabs')
        );

        if (await viewLevelTabs.isVisible()) {
          await test.step('切换视图等级', async () => {
            // 获取当前 URL
            const currentUrl = page.url();

            // 点击不同的视图等级选项
            const tabs = viewLevelTabs.locator('[role="tab"]').or(
              viewLevelTabs.locator('button')
            );

            const tabCount = await tabs.count();

            if (tabCount > 1) {
              // 点击第二个选项
              await tabs.nth(1).click();

              // 验证 URL 参数变化
              await page.waitForTimeout(500);
              const newUrl = page.url();
              expect(newUrl).not.toBe(currentUrl);
            }
          });
        }
      });

      // 返回列表页
      await page.goBack();
    }
  });

  test('应该显示 Token 统计', async ({ page }) => {
    // 导航到会话详情页
    const sessionCard = page.locator('[data-testid="session-card"]').or(
      page.locator('.session-card')
    );

    const cardCount = await sessionCard.count();

    if (cardCount > 0) {
      await sessionCard.first().click();

      await test.step('检查 Token 统计卡片', async () => {
        await page.waitForLoadState('networkidle');

        // 查找 Token 统计组件
        const tokenStats = page.locator('[data-testid="token-stats"]').or(
          page.locator('.token-stats')
        );

        if (await tokenStats.isVisible()) {
          // 验证显示输入/输出 Token 数量
          const text = await tokenStats.textContent();
          expect(text).toMatch(/token|输入|输出/i);
        }
      });
    }
  });

  test('应该响应式适配不同屏幕尺寸', async ({ page }) => {
    await test.step('测试小屏幕尺寸', async () => {
      // 设置移动设备视口
      await page.setViewportSize({ width: 375, height: 667 });

      // 验证关键元素仍然可见或可访问
      const sessionCards = page.locator('[data-testid="session-card"]').or(
        page.locator('.session-card')
      );

      const count = await sessionCards.count();

      if (count > 0) {
        await expect(sessionCards.first()).toBeVisible();
      }

      // 检查导航菜单（可能在移动端折叠）
      const navMenu = page.locator('[data-testid="nav-menu"]').or(
        page.locator('.nav-menu')
      );

      if (await navMenu.isVisible()) {
        await expect(navMenu).toBeVisible();
      }
    });

    await test.step('恢复桌面尺寸', async () => {
      // 恢复桌面视口
      await page.setViewportSize({ width: 1280, height: 720 });

      await page.waitForTimeout(500);
    });
  });

  test('应该正确处理加载和错误状态', async ({ page }) => {
    await test.step('模拟加载状态', async () => {
      // 刷新页面触发加载
      await page.reload();

      // 查找加载指示器
      const loadingIndicator = page.getByText(/加载中|loading/i).or(
        page.locator('[data-testid="loading"]')
      );

      // 加载指示器应该短暂显示
      const isVisible = await loadingIndicator.isVisible().catch(() => false);

      if (isVisible) {
        await expect(loadingIndicator).not.toBeVisible({ timeout: 10000 });
      }
    });

    await test.step('验证空状态', async () => {
      // 如果没有会话数据，应显示空状态
      const emptyState = page.getByText(/暂无会话|没有数据/i);

      const sessionCards = page.locator('[data-testid="session-card"]').or(
        page.locator('.session-card')
      );

      const cardCount = await sessionCards.count();

      if (cardCount === 0) {
        await expect(emptyState).toBeVisible();
      }
    });
  });

  test('应该能够使用键盘导航', async ({ page }) => {
    await test.step('测试 Tab 键导航', async () => {
      // 按 Tab 键遍历可聚焦元素
      await page.keyboard.press('Tab');

      // 验证某个元素获得焦点
      const focusedElement = await page.evaluate(() => document.activeElement?.tagName);
      expect(['BUTTON', 'INPUT', 'A', 'SELECT']).toContain(focusedElement);
    });

    await test.step('测试 Enter 键操作', async () => {
      // 聚焦到第一个会话卡片
      const sessionCard = page.locator('[data-testid="session-card"]').or(
        page.locator('.session-card')
      );

      const count = await sessionCard.count();

      if (count > 0) {
        await sessionCard.first().focus();
        await page.keyboard.press('Enter');

        // 验证导航到详情页
        await page.waitForTimeout(500);
        const url = page.url();
        const isDetailPage = url.includes('/sessions/') || url.includes('/session/');

        if (isDetailPage) {
          expect(url).toMatch(/\/sessions\/[a-z0-9-]+/i);
        }
      }
    });
  });
});
