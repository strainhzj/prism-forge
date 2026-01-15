# PowerShell 脚本 - 重新构建生产版本

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "PrismForge 生产版本构建工具" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 清理旧文件
Write-Host "[1/4] 正在清理旧文件..." -ForegroundColor Yellow

if (Test-Path "dist") {
    Remove-Item -Recurse -Force "dist"
    Write-Host "  ✓ 已清除 dist 目录" -ForegroundColor Green
}

if (Test-Path "src-tauri\target\release") {
    Remove-Item -Recurse -Force "src-tauri\target\release"
    Write-Host "  ✓ 已清除旧的构建文件" -ForegroundColor Green
}

if (Test-Path "node_modules\.vite") {
    Remove-Item -Recurse -Force "node_modules\.vite"
    Write-Host "  ✓ 已清除 Vite 缓存" -ForegroundColor Green
}

# 步骤 2: 构建前端
Write-Host ""
Write-Host "[2/4] 正在构建前端..." -ForegroundColor Yellow
npm run build

if ($LASTEXITCODE -ne 0) {
    Write-Host "  ✗ 前端构建失败" -ForegroundColor Red
    pause
    exit 1
}

Write-Host "  ✓ 前端构建成功" -ForegroundColor Green

# 步骤 3: 构建 Tauri 应用
Write-Host ""
Write-Host "[3/4] 正在构建 Tauri 应用..." -ForegroundColor Yellow
Write-Host "  这可能需要几分钟时间，请耐心等待..." -ForegroundColor Gray
npm run tauri build

if ($LASTEXITCODE -ne 0) {
    Write-Host "  ✗ Tauri 构建失败" -ForegroundColor Red
    pause
    exit 1
}

Write-Host "  ✓ Tauri 构建成功" -ForegroundColor Green

# 步骤 4: 显示输出位置
Write-Host ""
Write-Host "[4/4] 构建完成！" -ForegroundColor Green
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "生成的文件位置：" -ForegroundColor Cyan
Write-Host ""
Write-Host "  可执行文件:" -ForegroundColor White
Write-Host "  src-tauri\target\release\prism-forge.exe" -ForegroundColor Yellow
Write-Host ""
Write-Host "  安装程序:" -ForegroundColor White
Write-Host "  src-tauri\target\release\bundle\msi\*" -ForegroundColor Yellow
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "提示：" -ForegroundColor Cyan
Write-Host "1. 双击 prism-forge.exe 运行应用" -ForegroundColor White
Write-Host "2. 在应用中按 F12 可以打开开发者工具" -ForegroundColor White
Write-Host "3. 查看控制台确认是否加载了 SessionDetailPageV2" -ForegroundColor White
Write-Host "4. 右上角应该有红色标记 '✅ V2 NEW UI'" -ForegroundColor White
Write-Host ""
Write-Host "按任意键退出..." -ForegroundColor Gray
pause
