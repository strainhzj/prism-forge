# PowerShell 脚本 - 启动 Tauri 开发模式

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "PrismForge Tauri 开发模式启动工具" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 杀掉占用端口的进程
Write-Host "[1/3] 正在停止占用端口 1420 的进程..." -ForegroundColor Yellow

# 查找占用端口 1420 的进程
$port = 1420
$process = Get-NetTCPConnection -LocalPort $port -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess -ErrorAction SilentlyContinue

if ($process) {
    Write-Host "  发现占用端口 $port 的进程 PID: $process" -ForegroundColor Red
    Stop-Process -Id $process -Force
    Write-Host "  ✓ 已停止进程 $process" -ForegroundColor Green
    Start-Sleep -Seconds 2
} else {
    Write-Host "  - 端口 $port 未被占用" -ForegroundColor Gray
}

# 步骤 2: 清除 Vite 缓存
Write-Host ""
Write-Host "[2/3] 正在清除 Vite 缓存..." -ForegroundColor Yellow

if (Test-Path "node_modules\.vite") {
    Remove-Item -Recurse -Force "node_modules\.vite"
    Write-Host "  ✓ 已清除 Vite 缓存" -ForegroundColor Green
} else {
    Write-Host "  - Vite 缓存目录不存在" -ForegroundColor Gray
}

if (Test-Path "dist") {
    Remove-Item -Recurse -Force "dist"
    Write-Host "  ✓ 已清除旧的构建文件" -ForegroundColor Green
}

# 步骤 3: 启动 Tauri 开发模式
Write-Host ""
Write-Host "[3/3] 正在启动 Tauri 开发模式..." -ForegroundColor Yellow
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "提示：" -ForegroundColor Cyan
Write-Host "1. 应用窗口会自动打开" -ForegroundColor White
Write-Host "2. 在应用中按 F12 可以打开开发者工具" -ForegroundColor White
Write-Host "3. 查看控制台确认是否加载了 SessionDetailPageV2" -ForegroundColor White
Write-Host "4. 右上角应该有红色标记 '✅ V2 NEW UI'" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "正在启动... (按 Ctrl+C 停止)" -ForegroundColor Green
Write-Host ""

npm run tauri dev
