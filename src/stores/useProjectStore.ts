/**
 * Project Store - 项目管理
 *
 * 复用 useSessionStore 的 monitored_directories 数据
 * 提供项目切换、会话文件选择等功能
 */

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import { useMemo } from 'react';
import type {
  MonitoredDirectory,
  SessionFileInfo
} from '@/stores/useSessionStore';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[ProjectStore] ${action}`, ...args);
  }
}

// ==================== 类型定义 ====================

/**
 * 项目状态接口
 */
interface ProjectState {
  // 当前选中的项目
  currentProject: MonitoredDirectory | null;

  // 当前跟踪的会话文件路径
  currentSessionFile: string | null;

  // 项目列表（从 monitored_directories 同步）
  projects: MonitoredDirectory[];

  // 加载状态
  loading: boolean;
  error: string | null;

  // Actions
  fetchProjects: () => Promise<void>;
  setCurrentProject: (project: MonitoredDirectory | null) => void;
  setCurrentSessionFile: (filePath: string | null) => void;
  clearError: () => void;

  // 项目管理操作（复用 useSessionStore）
  addProject: (path: string, name: string) => Promise<MonitoredDirectory>;
  removeProject: (id: number) => Promise<void>;
  syncProjects: () => Promise<void>;

  // 会话文件操作
  getSessionFiles: (projectPath: string, includeAgent?: boolean) => Promise<SessionFileInfo[]>;
  getLatestSessionFile: (projectPath: string) => Promise<string | null>;
}

// ==================== 辅助函数 ====================

/**
 * 从路径中提取目录名称
 */
function extractDirectoryName(path: string): string {
  const normalizedPath = path.replace(/\\/g, '/');
  const parts = normalizedPath.split('/').filter(Boolean);
  return parts[parts.length - 1] || path;
}

/**
 * 解析错误信息
 */
function parseError(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

// ==================== Store 实现 ====================

export const useProjectStore = create<ProjectState>()(
  persist(
    immer((set, get) => ({
      // 初始状态
      currentProject: null,
      currentSessionFile: null,
      projects: [],
      loading: false,
      error: null,

      // 获取项目列表（从 monitored_directories 同步）
      fetchProjects: async () => {
        debugLog('fetchProjects', 'start');
        set((state) => {
          state.loading = true;
          state.error = null;
        });

        try {
          const directories = await invoke<MonitoredDirectory[]>('get_monitored_directories');
          debugLog('fetchProjects', 'success', directories.length);

          set((state) => {
            state.projects = directories;
            state.loading = false;
          });

          // 如果没有当前项目但有项目列表，自动选择第一个
          const { currentProject } = get();
          if (!currentProject && directories.length > 0) {
            // 优先选择活跃的项目
            const firstActive = directories.find(d => d.is_active);
            const projectToSelect = firstActive || directories[0];
            set((state) => {
              state.currentProject = projectToSelect;
            });
            debugLog('fetchProjects', 'auto select project', projectToSelect.name);
          }
        } catch (error) {
          debugLog('fetchProjects', 'error', error);
          set((state) => {
            state.error = `获取项目列表失败: ${parseError(error)}`;
            state.loading = false;
          });
        }
      },

      // 设置当前项目
      setCurrentProject: (project) => {
        debugLog('setCurrentProject', project?.name || 'null');
        set((state) => {
          state.currentProject = project;
          // 切换项目时清空当前会话文件
          state.currentSessionFile = null;
        });
      },

      // 设置当前会话文件
      setCurrentSessionFile: (filePath) => {
        debugLog('setCurrentSessionFile', filePath);
        set((state) => {
          state.currentSessionFile = filePath;
        });
      },

      // 清除错误
      clearError: () => {
        set((state) => {
          state.error = null;
        });
      },

      // 添加项目（复用 add_monitored_directory）
      addProject: async (path, name) => {
        debugLog('addProject', 'start', { path, name });
        set((state) => {
          state.loading = true;
          state.error = null;
        });

        try {
          const newDirectory = await invoke<MonitoredDirectory>('add_monitored_directory', {
            path,
            name,
          });
          debugLog('addProject', 'success');

          set((state) => {
            state.projects.push(newDirectory);
            state.loading = false;
          });

          return newDirectory;
        } catch (error) {
          debugLog('addProject', 'error', error);
          set((state) => {
            state.error = `添加项目失败: ${parseError(error)}`;
            state.loading = false;
          });
          throw error;
        }
      },

      // 删除项目（复用 remove_monitored_directory）
      removeProject: async (id) => {
        debugLog('removeProject', 'start', id);
        set((state) => {
          state.loading = true;
          state.error = null;
        });

        try {
          await invoke('remove_monitored_directory', { id });
          debugLog('removeProject', 'success');

          set((state) => {
            state.projects = state.projects.filter((p) => p.id !== id);
            // 如果删除的是当前项目，清空当前项目
            if (state.currentProject?.id === id) {
              state.currentProject = null;
              state.currentSessionFile = null;
            }
            state.loading = false;
          });
        } catch (error) {
          debugLog('removeProject', 'error', error);
          set((state) => {
            state.error = `删除项目失败: ${parseError(error)}`;
            state.loading = false;
          });
          throw error;
        }
      },

      // 同步项目列表（重新获取）
      syncProjects: async () => {
        await get().fetchProjects();
      },

      // 获取项目的会话文件列表（复用 get_sessions_by_monitored_directory）
      getSessionFiles: async (projectPath, includeAgent = false) => {
        debugLog('getSessionFiles', 'start', { projectPath, includeAgent });

        try {
          const files = await invoke<SessionFileInfo[]>('get_sessions_by_monitored_directory', {
            monitoredPath: projectPath,
            includeAgent,
            limit: 100, // 获取更多文件
            offset: 0,
          });
          debugLog('getSessionFiles', 'success', files.length);
          return files;
        } catch (error) {
          debugLog('getSessionFiles', 'error', error);
          throw error;
        }
      },

      // 获取项目最新的会话文件（复用 get_latest_session_path）
      getLatestSessionFile: async (projectPath) => {
        debugLog('getLatestSessionFile', 'start', projectPath);

        try {
          const latestPath = await invoke<string>('get_latest_session_path');
          debugLog('getLatestSessionFile', 'success', latestPath);
          return latestPath;
        } catch (error) {
          debugLog('getLatestSessionFile', 'error', error);
          return null;
        }
      },
    })),
    {
      name: 'prism-forge-project-storage',
      // 持久化当前项目ID和会话文件路径
      partialize: (state) => ({
        currentProjectId: state.currentProject?.id,
        currentSessionFile: state.currentSessionFile,
      }),
      // 从持久化数据恢复
      onRehydrateStorage: () => (state) => {
        if (state) {
          debugLog('onRehydrateStorage', 'restored state', {
            currentProjectId: state.currentProjectId,
            currentSessionFile: state.currentSessionFile,
          });
          // 注意：currentProject 需要从 projects 列表中重新查找
          // 这在 fetchProjects 中会自动处理
        }
      },
    }
  )
);

// ==================== 便捷 Hooks ====================

/**
 * 获取当前项目
 */
export const useCurrentProject = () => useProjectStore((state) => state.currentProject);

/**
 * 获取项目列表
 */
export const useProjects = () => useProjectStore((state) => state.projects);

/**
 * 获取当前会话文件
 */
export const useCurrentSessionFile = () => useProjectStore((state) => state.currentSessionFile);

/**
 * 获取项目操作（稳定引用）
 */
export const useProjectActions = () => {
  return useMemo(() => {
    const store = useProjectStore.getState();
    return {
      fetchProjects: store.fetchProjects,
      setCurrentProject: store.setCurrentProject,
      setCurrentSessionFile: store.setCurrentSessionFile,
      clearError: store.clearError,
      addProject: store.addProject,
      removeProject: store.removeProject,
      syncProjects: store.syncProjects,
      getSessionFiles: store.getSessionFiles,
      getLatestSessionFile: store.getLatestSessionFile,
    };
  }, []);
};

/**
 * 获取项目加载状态
 */
export const useProjectLoading = () => useProjectStore((state) => state.loading);

/**
 * 获取项目错误信息
 */
export const useProjectError = () => useProjectStore((state) => state.error);
