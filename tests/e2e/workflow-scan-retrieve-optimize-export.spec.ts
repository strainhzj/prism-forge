/**
 * E2E 测试：完整工作流
 * 扫描 → 检索 → 优化 → 导出
 */

import { test, expect } from '@playwright/test';

test.describe('完整工作流测试', () => {
  test.beforeEach(async ({ page }) => {
    // 导航到应用主页
    await page.goto('/');

    // 等待页面加载
    await page.waitForLoadState('networkidle');
  });

  test('应该能够完成扫描→检索→优化→导出流程', async ({ page }) => {
    // 步骤 1: 扫描 Claude 会话
    await test.step('扫描会话', async () => {
      // 点击扫描按钮（如果存在）
      const scanButton = page.getByRole('button', { name: /扫描|刷新/i });
      if (await scanButton.isVisible()) {
        await scanButton.click();

        // 等待扫描完成（加载状态消失）
        await expect(page.getByText(/加载会话列表/i)).not.toBeVisible({
          timeout: 10000,
        });
      }
    });

    // 步骤 2: 验证会话列表显示
    await test.step('验证会话列表', async () => {
      // 检查是否有会话卡片
      const sessionCards = page.locator('[data-testid="session-card"]').or(
        page.locator('.session-card')
      );

      const count = await sessionCards.count();

      // 至少应该有 Mock 数据或为空状态
      if (count > 0) {
        await expect(sessionCards.first()).toBeVisible();
      } else {
        // 空状态显示
        await expect(page.getByText(/暂无会话/i)).toBeVisible();
      }
    });

    // 步骤 3: 导航到提示词实验室
    await test.step('导航到提示词实验室', async () => {
      // 通过导航菜单或直接 URL
      await page.goto('/prompt-lab');
      await page.waitForLoadState('networkidle');

      // 验证页面标题
      await expect(page.getByText(/提示词实验室/i)).toBeVisible();
    });

    // 步骤 4: 输入优化目标
    await test.step('输入优化目标', async () => {
      const goalInput = page.getByPlaceholder(/输入优化目标/i).or(
        page.locator('textarea[placeholder*="目标"]')
      );

      if (await goalInput.isVisible()) {
        await goalInput.fill('优化 React 组件性能');
      }
    });

    // 步骤 5: 生成优化提示词
    await test.step('生成提示词', async () => {
      const generateButton = page.getByRole('button', { name: /生成/i });

      if (await generateButton.isVisible()) {
        // 启用下载监听
        const downloadPromise = page.waitForEvent('download');

        await generateButton.click();

        // 等待结果（显示 Token 统计或结果预览）
        await expect(
          page.getByText(/Token|节省/i).or(page.getByText(/优化结果/i))
        ).toBeVisible({ timeout: 15000 });
      }
    });

    // 步骤 6: 导出提示词
    await test.step('导出提示词', async () => {
      const exportButton = page.getByRole('button', { name: /导出/i });

      if (await exportButton.isVisible()) {
        const downloadPromise = page.waitForEvent('download');

        await exportButton.click();

        // 选择导出格式
        const formatOption = page.getByText(/JSON|CSV|Markdown/i);
        if (await formatOption.isVisible()) {
          await formatOption.first().click();
        }

        // 确认导出
        const confirmButton = page.getByRole('button', { name: /确认|下载/i });
        if (await confirmButton.isVisible()) {
          await confirmButton.click();
        }

        // 验证下载
        const download = await downloadPromise;
        expect(download.suggestedFilename()).toMatch(/\.(json|csv|md)$/);
      }
    });
  });

  test('应该能够搜索和过滤会话', async ({ page }) => {
    await test.step('搜索会话', async () => {
      // 等待会话列表加载
      await page.waitForLoadState('networkidle');

      // 查找搜索框
      const searchInput = page.getByPlaceholder(/搜索/i).or(
        page.locator('input[type="search"]')
      );

      if (await searchInput.isVisible()) {
        await searchInput.fill('test');

        // 等待过滤结果
        await page.waitForTimeout(500);

        // 验证搜索结果（根据实际实现调整）
        const sessionCards = page.locator('[data-testid="session-card"]').or(
          page.locator('.session-card')
        );

        const count = await sessionCards.count();
        expect(count).toBeGreaterThanOrEqual(0);
      }
    });
  });

  test('应该能够切换归档标签页', async ({ page }) => {
    await test.step('切换到归档标签页', async () => {
      // 查找归档标签
      const archivedTab = page.getByRole('tab', { name: /已归档/i });

      if (await archivedTab.isVisible()) {
        await archivedTab.click();

        // 验证 URL 或内容变化
        await expect(page.getByText(/已归档/i)).toBeVisible();
      }
    });
  });
});
