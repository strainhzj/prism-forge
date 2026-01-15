@echo off
echo ========================================
echo 快速测试脚本 - 验证新 UI
echo ========================================
echo.

echo [步骤 1] 检查是否有运行中的 Tauri 应用...
tasklist | findstr "prism-forge.exe"
if errorlevel 1 (
    echo     没有发现运行中的 prism-forge.exe
) else (
    echo     警告：发现 prism-forge.exe 正在运行
    echo     请先关闭应用，然后重新运行此脚本
    pause
    exit /b 1
)

echo.
echo [步骤 2] 检查端口占用...
netstat -ano | findstr ":1420"
if errorlevel 1 (
    echo     端口 1420 未被占用
) else (
    echo     错误：端口 1420 被占用！
    echo     请运行：杀进程并启动.bat
    pause
    exit /b 1
)

echo.
echo [步骤 3] 清除缓存...
if exist "node_modules\.vite" (
    rd /s /q "node_modules\.vite"
    echo     已清除 Vite 缓存
)

echo.
echo [步骤 4] 启动开发模式...
echo.
echo ========================================
echo 重要提示：
echo ========================================
echo.
echo 应用启动后：
echo.
echo 1. 应用窗口会自动打开
echo.
echo 2. 在应用中按 F12 打开开发者工具
echo.
echo 3. 点击 "Console" 标签，查找以下日志：
echo    🚀 [SessionDetailPageV2] 组件已加载！！！
echo.
echo 4. 查看右上角是否有红色标记：
echo    ✅ V2 NEW UI
echo.
echo 5. 如果看到红色标记，说明新 UI 成功加载！
echo.
echo ========================================
echo.
echo 按任意键继续启动...
pause >nul

echo.
echo 正在启动...
npm run tauri dev
