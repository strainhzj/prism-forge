import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, useLocation } from "react-router-dom";
import { QueryClientProvider } from "@tanstack/react-query";
import App from "./App";
import Settings from "./pages/Settings";
import { SettingsPage } from "./pages/SettingsPage";
import { SessionsPage } from "./pages/SessionsPage";
import { SessionDetailPage } from "./pages/SessionDetailPage";
import { getQueryClient } from "./lib/query-client";
import "./index.css";

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
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <RouteDebugger>
          <Routes>
            <Route path="/" element={<App />} />
            <Route path="/settings" element={<Settings />} />
            <Route path="/settings/v2" element={<SettingsPage />} />
            <Route path="/sessions" element={<SessionsPage />} />
            <Route path="/sessions/:sessionId" element={<SessionDetailPage />} />
          </Routes>
        </RouteDebugger>
      </BrowserRouter>
    </QueryClientProvider>
  </React.StrictMode>,
);
