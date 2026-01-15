@echo off
REM PrismForge 重新构建工具 - 生成新的可执行文件

echo.
echo ========================================
echo PrismForge 生产版本构建工具
echo ========================================
echo.

echo [1/4] 正在清理旧文件...
if exist "dist" (
    rd /s /q "dist"
    echo     已清除 dist 目录
)

if exist "src-tauri\target\release" (
    rd /s /q "src-tauri\target\release"
    echo     已清除旧的构建文件
)

if exist "node_modules\.vite" (
    rd /s /q "node_modules\.vite"
    echo     已清除 Vite 缓存
)

echo.
echo [2/4] 正在构建前端...
call npm run build
if errorlevel 1 (
    echo.
    echo [错误] 前端构建失败！
    pause
    exit /b 1
)
echo     前端构建成功
echo.

echo [3/4] 正在构建 Tauri 应用...
echo     这可能需要几分钟时间，请耐心等待...
echo.
call npm run tauri build
if errorlevel 1 (
    echo.
    echo [错误] Tauri 构建失败！
    pause
    exit /b 1
)
echo     Tauri 构建成功
echo.

echo [4/4] 构建完成！
echo.
echo ========================================
echo 生成的文件位置：
echo.
echo   可执行文件:
echo   src-tauri\target\release\prism-forge.exe
echo.
echo   安装程序:
echo   src-tauri\target\release\bundle\msi\*
echo.
echo ========================================
echo.
echo 提示：
echo 1. 双击 prism-forge.exe 运行应用
echo 2. 在应用中按 F12 可以打开开发者工具
echo 3. 查看控制台确认是否加载了 SessionDetailPageV2
echo 4. 右上角应该有红色标记 '✅ V2 NEW UI'
echo.
echo 按任意键退出...
pause >nul
