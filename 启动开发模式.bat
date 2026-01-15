@echo off
REM PrismForge 启动工具 - 杀掉占用端口的进程并启动开发模式

echo.
echo ========================================
echo PrismForge 开发模式启动工具
echo ========================================
echo.

echo [1/3] 正在停止占用端口 1420 的进程...

REM 查找并杀掉占用端口 1420 的进程
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :1420') do (
    echo 发现占用端口的进程 PID: %%a
    taskkill /PID %%a /F >nul 2>&1
    if errorlevel 1 (
        echo 进程 %%a 已停止或无法访问
    ) else (
        echo     已停止进程 %%a
    )
)

echo.
echo [2/3] 正在清除 Vite 缓存...
if exist "node_modules\.vite" (
    rd /s /q "node_modules\.vite"
    echo     已清除 Vite 缓存
) else (
    echo     Vite 缓存目录不存在
)

if exist "dist" (
    rd /s /q "dist"
    echo     已清除旧的构建文件
)

echo.
echo [3/3] 正在启动 Tauri 开发模式...
echo.
echo ========================================
echo 提示：
echo 1. 应用窗口会自动打开
echo 2. 在应用中按 F12 可以打开开发者工具
echo 3. 查看控制台确认是否加载了 SessionDetailPageV2
echo 4. 右上角应该有红色标记 '✅ V2 NEW UI'
echo ========================================
echo.
echo 正在启动... (按 Ctrl+C 停止)
echo.

npm run tauri dev

pause
