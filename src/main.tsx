import React, { lazy, Suspense } from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, useLocation } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import App from "./App";
import { getQueryClient } from "./lib/query-client";
import { Loading } from "./components/ui/loading";
import { ThemeProvider } from "./contexts/ThemeContext";
import "./i18n"; // 初始化 i18n
import "./index.css";

// 代码分割：路由级懒加载
const Settings = lazy(() => import("./pages/Settings"));
const SettingsPage = lazy(() =>
  import("./pages/SettingsPage").then(m => ({ default: m.SettingsPage }))
);
const SessionsPage = lazy(() =>
  import("./pages/SessionsPage").then(m => ({ default: m.SessionsPage }))
);
const PromptLab = lazy(() =>
  import("./pages/PromptLab").then(m => ({ default: m.PromptLab }))
);
const DiagnosticPage = lazy(() =>
  import("./pages/DiagnosticPage").then(m => ({ default: m.DiagnosticPage }))
);

// 加载中组件
const RouteLoadingFallback = () => (
  <div className="flex items-center justify-center min-h-screen">
    <Loading text="加载页面..." />
  </div>
);

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

// 路由调试组件
const RouteDebugger: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const location = useLocation();
  
  React.useEffect(() => {
    if (DEBUG) {
      console.log('[Router] Location changed:', location.pathname);
    }
  }, [location]);
  
  return <>{children}</>;
};

// 全局错误处理
if (DEBUG) {
  window.addEventListener('error', (event) => {
    console.error('[Global] Uncaught error:', event.error);
  });
  
  window.addEventListener('unhandledrejection', (event) => {
    console.error('[Global] Unhandled promise rejection:', event.reason);
  });
  
  console.log('[Main] App starting in DEBUG mode');
}

const queryClient = getQueryClient();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="system">
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <RouteDebugger>
            <Suspense fallback={<RouteLoadingFallback />}>
              <Routes>
                <Route path="/" element={<App />} />
                <Route path="/settings" element={<Settings />} />
                <Route path="/settings/v2" element={<SettingsPage />} />
                <Route path="/sessions" element={<SessionsPage />} />
                <Route path="/prompt-lab" element={<PromptLab />} />
                <Route path="/diagnostic" element={<DiagnosticPage />} />
              </Routes>
            </Suspense>
          </RouteDebugger>
        </BrowserRouter>
      </QueryClientProvider>
    </ThemeProvider>
  </React.StrictMode>,
);
