# PrismForge 端到端测试计划

**版本**: v1.0.0
**日期**: 2025-01-02
**测试框架**: Playwright

---

## 1. 测试范围

### 1.1 核心功能测试

| 功能模块 | 测试场景 | 优先级 |
|---------|---------|--------|
| 会话扫描 | 扫描 Claude 会话文件 | 高 |
| 向量检索 | RAG 检索相关会话 | 高 |
| 提示词优化 | 生成优化提示词 | 高 |
| 数据导出 | 导出 JSON/CSV/Markdown | 中 |
| Provider 管理 | 配置和测试连接 | 高 |
| 会话管理 | 评分、标签、归档 | 中 |
| 实时监控 | 自动刷新会话列表 | 中 |

### 1.2 测试环境

- **开发环境**: http://localhost:1420
- **测试数据**: Mock Claude 会话文件
- **测试浏览器**: Chromium (Desktop)

---

## 2. 测试场景

### 2.1 完整工作流：扫描 → 检索 → 优化 → 导出

**测试用例**: `workflow-scan-retrieve-optimize-export.spec.ts`

**步骤**:
1. 启动应用，导航到会话列表页
2. 点击"扫描"按钮，扫描 Claude 会话目录
3. 验证会话列表显示扫描结果
4. 进入提示词实验室
5. 选择一个会话，输入优化目标
6. 点击"生成"按钮
7. 验证生成的提示词和 Token 统计
8. 点击"导出"按钮
9. 选择导出格式（JSON/CSV/Markdown）
10. 验证文件下载成功

**预期结果**:
- ✅ 扫描成功，会话列表正确显示
- ✅ 检索返回相关会话
- ✅ 优化生成提示词，显示节省 Token 百分比
- ✅ 导出文件格式正确，内容完整

---

### 2.2 Provider 管理工作流

**测试用例**: `settings-provider.spec.ts`

**步骤**:
1. 导航到设置页面
2. 点击"添加 Provider"按钮
3. 选择提供商类型（OpenAI/Anthropic/Ollama/xAI）
4. 输入名称和 API Key
5. 点击"测试连接"
6. 验证连接成功
7. 保存 Provider
8. 验证 Provider 显示在列表中
9. 编辑 Provider 配置
10. 删除 Provider

**预期结果**:
- ✅ 所有提供商类型正确配置
- ✅ API Key 验证和测试连接正常
- ✅ CRUD 操作正常工作

---

### 2.3 会话管理工作流

**测试用例**: `session-management.spec.ts`

**步骤**:
1. 在会话列表页
2. 对一个会话进行评分（1-5 星）
3. 验证评分显示正确
4. 编辑会话标签
5. 验证标签更新成功
6. 归档会话
7. 切换到"已归档"标签页
8. 验证归档会话显示
9. 取消归档
10. 验证会话返回主列表

**预期结果**:
- ✅ 评分正确保存和显示
- ✅ 标签编辑功能正常
- ✅ 归档/取消归档正常工作

---

### 2.4 实时监控工作流

**测试用例**: `realtime-monitoring.spec.ts`

**步骤**:
1. 打开会话列表页
2. 修改外部 Claude 会话文件
3. 等待 2-5 秒
4. 验证自动刷新指示器显示
5. 验证会话列表更新
6. 手动点击"刷新"按钮
7. 验证立即刷新

**预期结果**:
- ✅ 自动检测文件变更
- ✅ 显示刷新指示器
- ✅ 自动更新会话列表

---

### 2.5 代码 Diff 显示

**测试用例**: `code-diff-view.spec.ts`

**步骤**:
1. 进入一个包含代码变更的会话详情页
2. 找到代码变更消息节点
3. 查看 Diff 视图（并排模式）
4. 切换到统一模式
5. 验证代码高亮正确
6. 测试大文件 Diff（1000+ 行）
7. 验证截断和分页功能

**预期结果**:
- ✅ Diff 正确显示代码变更
- ✅ 模式切换正常
- ✅ 大文件性能优化生效

---

## 3. 测试数据准备

### 3.1 Mock 会话数据

在 `tests/e2e/fixtures/` 目录下准备：
- `valid_session.json` - 有效的 Claude 会话文件
- `large_session.json` - 大型会话文件（1000+ 消息）
- `code_change_session.json` - 包含代码变更的会话

### 3.2 环境变量

创建 `.env.test` 文件：
```env
TAURI_PATH=tests/e2e/fixtures
CLAUDE_SESSIONS_PATH=tests/e2e/fixtures/sessions
```

---

## 4. 测试执行

### 4.1 运行所有测试

```bash
# 安装浏览器
npx playwright install chromium

# 运行所有 E2E 测试
npm run test:e2e

# 运行特定测试文件
npx playwright test workflow-scan-retrieve-optimize-export.spec.ts

# 调试模式
npx playwright test --debug

# 生成报告
npx playwright show-report
```

### 4.2 CI/CD 集成

**GitHub Actions 示例**:

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      - run: npm ci
      - run: npx playwright install --with-deps
      - run: npm run test:e2e
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: playwright-report
          path: playwright-report/
```

---

## 5. 验收标准

### 5.1 AC1: 所有核心功能测试通过

- [ ] 扫描 → 检索 → 优化 → 导出
- [ ] Provider 管理工作流
- [ ] 会话管理工作流
- [ ] 实时监控工作流
- [ ] 代码 Diff 显示

### 5.2 AC2: 测试覆盖率 > 80%

使用 Playwright 内置覆盖率报告：
```bash
npx playwright test --coverage
```

目标：
- 语句覆盖率 > 80%
- 分支覆盖率 > 75%
- 函数覆盖率 > 80%

### 5.3 AC3: 无阻塞性 bug

阻塞性 bug 定义：
- 导致应用崩溃
- 核心功能无法使用
- 数据丢失或损坏

---

## 6. 已知问题和限制

### 6.1 Tauri API 测试

由于 Playwright 无法直接调用 Tauri API，需要：
- 使用 Mock 数据模拟后端响应
- 或者使用 Tauri 的测试工具链

### 6.2 文件系统访问

E2E 测试环境中：
- 无法访问真实的用户 Claude 目录
- 需要使用 Mock 文件系统

### 6.3 网络请求

外部 API 调用（OpenAI、Anthropic）需要：
- 使用 Mock Service Worker (MSW)
- 或者配置测试环境专用的 API Key

---

## 7. 维护和更新

### 7.1 测试用例更新

当功能变更时：
1. 更新对应的测试用例
2. 更新测试计划文档
3. 运行完整测试套件验证

### 7.2 测试数据维护

定期更新 Mock 数据以匹配：
- 最新的 Claude 会话格式
- 新增的功能字段
- 边界情况和异常数据

---

**文档版本**: v1.0.0
**最后更新**: 2025-01-02
**维护者**: QA Team
