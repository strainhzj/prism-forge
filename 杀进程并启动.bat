@echo off
echo 正在查找占用端口 1420 的进程...

REM 查找占用端口的进程并杀掉
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :1420') do (
    echo.
    echo 发现进程 PID: %%a
    taskkill /PID %%a /F
    if errorlevel 1 (
        echo 进程 %%a 无法停止（可能已停止）
    ) else (
        echo 成功停止进程 %%a
    )
)

echo.
echo 等待 2 秒...
timeout /t 2 /nobreak >nul

echo.
echo 正在启动 Tauri 开发模式...
echo.
npm run tauri dev

pause
