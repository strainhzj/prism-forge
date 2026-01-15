@echo off
REM PrismForge UI 更新 - 清除缓存并重启
REM 使用方法：双击此文件或在 PowerShell/CMD 中运行

echo.
echo ========================================
echo PrismForge UI 缓存清理和重启工具
echo ========================================
echo.

echo [1/5] 停止所有 Node 进程...
taskkill /F /IM node.exe 2>nul
if %errorlevel% == 0 (
    echo     ✓ 已停止 Node 进程
) else (
    echo     - 未发现运行中的 Node 进程
)
timeout /t 2 /nobreak >nul

echo.
echo [2/5] 清除 Vite 缓存...
if exist "node_modules\.vite" (
    rd /s /q "node_modules\.vite"
    echo     ✓ 已清除 Vite 缓存
) else (
    echo     - Vite 缓存目录不存在
)

echo.
echo [3/5] 清除构建文件...
if exist "dist" (
    rd /s /q "dist"
    echo     ✓ 已清除构建文件
) else (
    echo     - 构建目录不存在
)

echo.
echo [4/5] 重新构建项目...
call npm run build
if %errorlevel% == 0 (
    echo     ✓ 构建成功
) else (
    echo     ✗ 构建失败，请检查错误信息
    pause
    exit /b 1
)

echo.
echo [5/5] 启动开发服务器...
echo.
echo ========================================
echo 提示：
echo 1. 服务器启动后，请在浏览器中访问
echo 2. 使用 Ctrl+Shift+R 硬刷新浏览器
echo 3. 或在隐私/无痕模式下打开
echo ========================================
echo.
echo 正在启动开发服务器...
echo.

call npm run tauri dev

pause
