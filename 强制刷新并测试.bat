@echo off
echo ========================================
echo 强制清除缓存并重新启动
echo ========================================
echo.

echo [1/5] 停止所有相关进程...
taskkill /F /IM node.exe >nul 2>&1
taskkill /F /IM prism-forge.exe >nul 2>&1
taskkill /F /IM cargo.exe >nul 2>&1
echo     已停止所有进程

echo.
echo [2/5] 清除所有缓存...
if exist "node_modules\.vite" (
    rd /s /q "node_modules\.vite"
    echo     已清除 Vite 缓存
)
if exist "dist" (
    rd /s /q "dist"
    echo     已清除旧的构建文件
)
if exist "src-tauri\target\debug" (
    rd /s /q "src-tauri\target\debug"
    echo     已清除 Debug 构建
)
echo     缓存清除完成

echo.
echo [3/5] 重新构建前端...
call npm run build
if errorlevel 1 (
    echo     构建失败！
    pause
    exit /b 1
)

echo.
echo [4/5] 启动 Tauri 开发模式...
echo.
echo ========================================
echo 关键提示：
echo ========================================
echo.
echo 1. 应用窗口打开后，立即按 F12
echo.
echo 2. 在开发者工具中，右键点击刷新按钮
echo.
echo 3. 选择"清空缓存并硬刷新页面"
echo.
echo 4. 或者按 Ctrl + Shift + R
echo.
echo 5. 查看 Console 是否有：
echo    🚀 [SessionDetailPageV2] 组件已加载
echo.
echo 6. 右上角应该有红色标记：✅ V2 NEW UI
echo.
echo ========================================
echo.

start npm run tauri dev

echo.
echo 开发模式已在后台启动...
echo.
timeout /t 10 /nobreak >nul
echo.
echo ========================================
echo 如果 10 秒后应用未打开，请检查：
echo 1. 是否有防火墙阻止
echo 2. 端口 1420 是否被占用
echo 3. 查看上方的错误信息
echo ========================================
echo.
pause
