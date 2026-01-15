@echo off
echo ========================================
echo PrismForge UI 诊断和修复工具
echo ========================================
echo.

echo [诊断 1] 检查当前运行的 EXE 位置
echo.
where prism-forge.exe
if errorlevel 1 (
    echo     系统路径中未找到 prism-forge.exe
) else (
    echo     找到：将显示完整路径
)

echo.
echo [诊断 2] 检查项目中的 EXE 文件
echo.
dir /s /b "src-tauri\target\release\*.exe" 2>nul
if errorlevel 1 (
    echo     未找到编译的 EXE 文件
)

echo.
echo [诊断 3] 检查 dist 目录内容
echo.
dir "dist\assets\*SessionDetail*.js" /b 2>nul
if errorlevel 1 (
    echo     未找到 SessionDetailPage 相关文件
) else (
    echo     找到以下文件：
    dir "dist\assets\*SessionDetail*.js" /b
)

echo.
echo ========================================
echo 修复步骤
echo ========================================
echo.

echo [步骤 1] 完全清理...
taskkill /F /IM node.exe >nul 2>&1
taskkill /F /IM prism-forge.exe >nul 2>&1
timeout /t 2 /nobreak >nul

if exist "node_modules\.vite" rd /s /q "node_modules\.vite"
if exist "dist" rd /s /q "dist"
if exist "src-tauri\target\debug" rd /s /q "src-tauri\target\debug"

echo     清理完成
echo.

echo [步骤 2] 重新构建...
call npm run build
if errorlevel 1 goto :error

echo.
echo [步骤 3] 启动 Tauri 开发模式...
echo.
echo ========================================
echo 重要：如何验证新 UI
echo ========================================
echo.
echo 方法 1 - 查看红色标记（最明显）:
echo   应用打开后，右上角应该有红色背景的
echo   "✅ V2 NEW UI" 标记
echo.
echo 方法 2 - 使用开发者工具:
echo   1. 在应用中按 Ctrl+Shift+I（不是 F12）
echo   2. 点击 "Console" 标签
echo   3. 查找：🚀 [SessionDetailPageV2]
echo   4. 如果看到此日志，说明新 UI 已加载！
echo.
echo 方法 3 - 查看 Network 标签:
echo   1. Ctrl+Shift+I 打开开发者工具
echo   2. 点击 "Network" 标签
echo   3. 刷新页面（Ctrl+R）
echo   4. 查找：SessionDetailPageV2-xxx.js
echo.
echo ========================================
echo.

start /wait npm run tauri dev
goto :end

:error
echo.
echo ========================================
echo 错误：构建失败！
echo ========================================
pause
exit /b 1

:end
echo.
echo 开发模式已停止
pause
