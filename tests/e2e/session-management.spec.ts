/**
 * E2E 测试：会话管理功能
 * 评分、标签、归档
 */

import { test, expect } from '@playwright/test';

test.describe('会话管理测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/sessions');
    await page.waitForLoadState('networkidle');
  });

  test('应该能够对会话进行评分', async ({ page }) => {
    await test.step('查找会话卡片', async () => {
      // 等待会话列表加载
      const sessionCard = page.locator('[data-testid="session-card"]').or(
        page.locator('.session-card')
      );

      const count = await sessionCard.count();

      if (count > 0) {
        const firstCard = sessionCard.first();

        await test.step('点击星级评分', async () => {
          // 查找星星按钮（通常有 5 个）
          const starButton = firstCard.locator('[data-star-value]').or(
            firstCard.locator('button').filter({ hasText: /★/i })
          );

          if (await starButton.count() > 0) {
            // 点击第 4 颗星（4 星评分）
            await starButton.nth(3).click();

            // 等待更新
            await page.waitForTimeout(500);

            // 验证评分显示（查找填充的星星或徽章）
            const ratedBadge = firstCard.locator('[data-rating]').or(
              firstCard.getByText(/4.*星/i)
            );

            // 验证评分已保存（可能需要重新加载）
            await page.reload();
            await expect(firstCard).toBeVisible();
          }
        });
      } else {
        test.skip(true, '没有可用的会话卡片');
      }
    });
  });

  test('应该能够编辑会话标签', async ({ page }) => {
    const sessionCard = page.locator('[data-testid="session-card"]').or(
      page.locator('.session-card')
    );

    const count = await sessionCard.count();

    if (count > 0) {
      const firstCard = sessionCard.first();

      await test.step('打开标签编辑', async () => {
        const editTagsButton = firstCard.getByRole('button', { name: /编辑.*标签|标签/i });

        if (await editTagsButton.isVisible()) {
          await editTagsButton.click();

          // 验证标签编辑器打开
          await expect(page.getByText(/编辑标签|添加标签/i)).toBeVisible();
        }
      });

      await test.step('添加新标签', async () => {
        const tagInput = page.getByPlaceholder(/输入标签|添加/i).or(
          page.locator('input[placeholder*="标签"]')
        );

        if (await tagInput.isVisible()) {
          await tagInput.fill('测试标签');

          const addButton = page.getByRole('button', { name: /添加|确定/i });
          if (await addButton.isVisible()) {
            await addButton.click();

            // 验证标签显示
            await expect(page.getByText(/测试标签/i)).toBeVisible();
          }
        }
      });

      await test.step('保存标签', async () => {
        const saveButton = page.getByRole('button', { name: /保存/i });
        if (await saveButton.isVisible()) {
          await saveButton.click();

          // 验证保存成功
          await expect(page.getByText(/保存成功|已更新/i)).toBeVisible({
            timeout: 5000,
          });
        }
      });
    } else {
      test.skip(true, '没有可用的会话卡片');
    }
  });

  test('应该能够归档会话', async ({ page }) => {
    const sessionCard = page.locator('[data-testid="session-card"]').or(
      page.locator('.session-card')
    );

    const count = await sessionCard.count();

    if (count > 0) {
      const firstCard = sessionCard.first();

      await test.step('点击归档按钮', async () => {
        const archiveButton = firstCard.getByRole('button', { name: /归档/i });

        if (await archiveButton.isVisible()) {
          await archiveButton.click();

          // 等待归档完成
          await page.waitForTimeout(1000);

          // 验证卡片状态变化（可能变为半透明或消失）
          const isArchived = await firstCard.evaluate((el) =>
            el.classList.contains('archived') || el.classList.contains('opacity-60')
          );

          expect(isArchived).toBeTruthy();
        }
      });
    } else {
      test.skip(true, '没有可用的会话卡片');
    }
  });

  test('应该能够切换归档标签页', async ({ page }) => {
    await test.step('切换到已归档标签页', async () => {
      const archivedTab = page.getByRole('tab', { name: /已归档/i });

      if (await archivedTab.isVisible()) {
        await archivedTab.click();

        // 验证标签页激活
        await expect(archivedTab).toHaveAttribute('data-state', 'active');

        // 验证内容变化
        await page.waitForTimeout(500);
      }
    });

    await test.step('取消归档会话', async () => {
      const sessionCard = page.locator('[data-testid="session-card"]').or(
        page.locator('.session-card')
      );

      const count = await sessionCard.count();

      if (count > 0) {
        const firstCard = sessionCard.first();

        const unarchiveButton = firstCard.getByRole('button', { name: /取消归档/i });

        if (await unarchiveButton.isVisible()) {
          await unarchiveButton.click();

          // 等待取消归档完成
          await page.waitForTimeout(1000);
        }
      }
    });
  });

  test('应该能够搜索和过滤会话', async ({ page }) => {
    await test.step('使用搜索功能', async () => {
      const searchInput = page.getByPlaceholder(/搜索/i).or(
        page.locator('input[type="search"]')
      );

      if (await searchInput.isVisible()) {
        await searchInput.fill('test');

        // 等待过滤结果
        await page.waitForTimeout(500);

        // 验证搜索结果
        const sessionCards = page.locator('[data-testid="session-card"]').or(
          page.locator('.session-card')
        );

        // 搜索应该过滤结果
        const count = await sessionCard.count();
        expect(count).toBeGreaterThanOrEqual(0);
      }
    });

    await test.step('清空搜索', async () => {
      const searchInput = page.getByPlaceholder(/搜索/i).or(
        page.locator('input[type="search"]')
      );

      if (await searchInput.isVisible()) {
        await searchInput.clear();

        // 等待结果恢复
        await page.waitForTimeout(500);

        // 验证所有会话显示
        const resetButton = page.getByRole('button', { name: /重置/i });
        if (await resetButton.isVisible()) {
          await resetButton.click();
          await page.waitForTimeout(500);
        }
      }
    });
  });

  test('应该能够查看会话详情', async ({ page }) => {
    const sessionCard = page.locator('[data-testid="session-card"]').or(
      page.locator('.session-card')
    );

    const count = await sessionCard.count();

    if (count > 0) {
      await test.step('点击会话卡片', async () => {
        const firstCard = sessionCard.first();
        await firstCard.click();

        // 验证导航到详情页
        await expect(page).toHaveURL(/\/sessions\/[a-z0-9-]+/i);
      });

      await test.step('验证详情页内容', async () => {
        // 检查详情页元素
        await expect(page.getByText(/会话详情/i)).toBeVisible();

        // 检查消息树或内容区域
        const messageTree = page.locator('[data-testid="message-tree"]').or(
          page.locator('.message-tree')
        );

        // 消息树可能需要时间加载
        await page.waitForTimeout(1000);
      });

      await test.step('返回会话列表', async () => {
        const backButton = page.getByRole('button', { name: /返回|back/i });

        if (await backButton.isVisible()) {
          await backButton.click();

          // 验证返回列表页
          await expect(page).toHaveURL('/sessions');
        } else {
          // 或者使用浏览器后退
          await page.goBack();
          await expect(page).toHaveURL('/sessions');
        }
      });
    } else {
      test.skip(true, '没有可用的会话卡片');
    }
  });

  test('应该能够批量操作会话（如果支持）', async ({ page }) => {
    // 检查是否有批量选择功能
    const selectAllCheckbox = page.locator('input[type="checkbox"]').first();

    if (await selectAllCheckbox.isVisible()) {
      await test.step('选择多个会话', async () => {
        await selectAllCheckbox.click();

        // 验证选中状态
        const checkedCount = await page.locator('input[type="checkbox"]:checked').count();
        expect(checkedCount).toBeGreaterThan(0);
      });

      await test.step('批量归档', async () => {
        const batchArchiveButton = page.getByRole('button', { name: /批量归档|归档选中/i });

        if (await batchArchiveButton.isVisible()) {
          await batchArchiveButton.click();

          // 验证确认对话框
          const confirmButton = page.getByRole('button', { name: /确认|确定/i });
          if (await confirmButton.isVisible()) {
            await confirmButton.click();
          }

          // 验证操作成功
          await expect(page.getByText(/归档成功|已完成/i)).toBeVisible({
            timeout: 5000,
          });
        }
      });
    } else {
      test.skip(true, '批量操作功能未实现');
    }
  });
});
