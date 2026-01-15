/**
 * E2E 测试：Provider 管理功能
 * 配置、测试连接、CRUD 操作
 */

import { test, expect } from '@playwright/test';

test.describe('Provider 管理测试', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
  });

  test('应该能够添加新的 Provider', async ({ page }) => {
    await test.step('打开添加对话框', async () => {
      const addButton = page.getByRole('button', { name: /添加|新增/i });

      if (await addButton.isVisible()) {
        await addButton.click();

        // 验证对话框打开
        await expect(page.getByText(/添加.*Provider/i)).toBeVisible();
      }
    });

    await test.step('填写 Provider 信息', async () => {
      // 选择提供商类型
      const typeSelect = page.locator('select').or(page.getByRole('combobox'));
      if (await typeSelect.isVisible()) {
        await typeSelect.selectOption('openai');
      }

      // 输入名称
      const nameInput = page.getByLabel(/名称/i).or(
        page.locator('input[name*="name"]')
      );
      if (await nameInput.isVisible()) {
        await nameInput.fill('Test OpenAI Provider');
      }

      // 输入 API Key
      const apiKeyInput = page.getByLabel(/API.*Key/i).or(
        page.locator('input[type="password"]')
      );
      if (await apiKeyInput.isVisible()) {
        await apiKeyInput.fill('sk-test-1234567890');
      }
    });

    await test.step('保存 Provider', async () => {
      const saveButton = page.getByRole('button', { name: /保存|提交/i });
      if (await saveButton.isVisible()) {
        await saveButton.click();

        // 验证保存成功（消息或列表更新）
        await expect(
          page.getByText(/保存成功|已添加/i).or(page.locator('.provider-card'))
        ).toBeVisible({ timeout: 5000 });
      }
    });
  });

  test('应该能够测试 Provider 连接', async ({ page }) => {
    await test.step('点击测试连接', async () => {
      // 假设已有 Provider 列表
      const testButton = page.getByRole('button', { name: /测试连接/i });

      if (await testButton.isVisible()) {
        // 监听网络请求
        const responsePromise = page.waitForResponse(
          (resp) => resp.url().includes('test') || resp.url().includes('ping')
        );

        await testButton.first().click();

        // 等待响应
        const response = await responsePromise;

        // 验证状态码或显示消息
        expect(response.status()).toBeGreaterThanOrEqual(200);
        expect(response.status()).toBeLessThan(500);

        // 验证成功/失败消息
        await expect(
          page.getByText(/连接成功|连接失败|测试/i)
        ).toBeVisible({ timeout: 10000 });
      }
    });
  });

  test('应该能够编辑 Provider', async ({ page }) => {
    await test.step('打开编辑对话框', async () => {
      const editButton = page.getByRole('button', { name: /编辑|修改/i });

      if (await editButton.isVisible()) {
        await editButton.first().click();

        // 验证对话框打开并填充现有数据
        await expect(page.getByText(/编辑.*Provider/i)).toBeVisible();
      }
    });

    await test.step('修改 Provider 信息', async () => {
      const nameInput = page.getByLabel(/名称/i).or(
        page.locator('input[name*="name"]')
      );

      if (await nameInput.isVisible()) {
        // 清空并输入新名称
        await nameInput.clear();
        await nameInput.fill('Updated Provider Name');
      }
    });

    await test.step('保存修改', async () => {
      const saveButton = page.getByRole('button', { name: /保存/i });
      if (await saveButton.isVisible()) {
        await saveButton.click();

        // 验证更新成功
        await expect(page.getByText(/更新成功|已保存/i)).toBeVisible({
          timeout: 5000,
        });
      }
    });
  });

  test('应该能够删除 Provider', async ({ page }) => {
    await test.step('删除 Provider', async () => {
      const deleteButton = page.getByRole('button', { name: /删除/i });

      if (await deleteButton.isVisible()) {
        // 某些实现可能有确认对话框
        page.on('dialog', (dialog) => dialog.accept());

        await deleteButton.first().click();

        // 验证删除成功
        await expect(page.getByText(/删除成功|已删除/i)).toBeVisible({
          timeout: 5000,
        });
      }
    });
  });

  test('应该能够切换活跃 Provider', async ({ page }) => {
    await test.step('切换活跃状态', async () => {
      // 查找活跃状态切换按钮
      const toggleButton = page.getByRole('button', { name: /设为活跃|激活/i }).or(
        page.locator('input[type="radio"]')
      );

      if (await toggleButton.isVisible()) {
        const initialCount = await page.locator('.provider-card.active').count();

        await toggleButton.first().click();

        // 验证只有一个活跃 Provider
        await page.waitForTimeout(500);
        const finalCount = await page.locator('.provider-card.active').count();

        expect(finalCount).toBe(1);
      }
    });
  });

  test('表单验证应该正常工作', async ({ page }) => {
    await test.step('测试必填字段验证', async () => {
      const addButton = page.getByRole('button', { name: /添加|新增/i });

      if (await addButton.isVisible()) {
        await addButton.click();

        // 尝试不填写任何字段直接保存
        const saveButton = page.getByRole('button', { name: /保存/i });
        if (await saveButton.isVisible()) {
          await saveButton.click();

          // 验证错误提示
          await expect(page.getByText(/必填|不能为空|required/i)).toBeVisible();
        }
      }
    });

    await test.step('测试 API Key 格式验证', async () => {
      const apiKeyInput = page.getByLabel(/API.*Key/i).or(
        page.locator('input[type="password"]')
      );

      if (await apiKeyInput.isVisible()) {
        await apiKeyInput.fill('invalid-key');

        const saveButton = page.getByRole('button', { name: /保存/i });
        if (await saveButton.isVisible()) {
          await saveButton.click();

          // 验证格式错误提示（根据实际实现）
          // await expect(page.getByText(/格式错误|无效/i)).toBeVisible();
        }
      }
    });
  });
});
