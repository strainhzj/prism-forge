/**
 * Session Store - 会话管理
 *
 * 使用 Zustand 管理会话列表状态，包括：
 * - sessions: 会话列表
 * - filteredSessions: 过滤后的会话
 * - projects: 项目列表（按项目分组）
 * - 各种操作（扫描、过滤、评分、标签等）
 */

import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { invoke } from '@tauri-apps/api/core';
import { useMemo } from 'react';

// ==================== 调试模式 ====================
const DEBUG = import.meta.env.DEV;

function debugLog(action: string, ...args: unknown[]) {
  if (DEBUG) {
    console.log(`[SessionStore] ${action}`, ...args);
  }
}

/**
 * 解析 Tauri 命令错误
 */
function parseError(error: unknown): string {
  if (typeof error === 'string') {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  if (error && typeof error === 'object') {
    if ('message' in error && typeof (error as { message: unknown }).message === 'string') {
      return (error as { message: string }).message;
    }
    try {
      return JSON.stringify(error);
    } catch {
      return String(error);
    }
  }
  return String(error);
}

// ==================== 类型定义 ====================

/**
 * 监控目录接口（与 Rust 后端 MonitoredDirectory 模型对应）
 */
export interface MonitoredDirectory {
  id?: number;
  path: string;
  name: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

/**
 * 会话状态接口（与 Rust 后端 Session 模型对应）
 */
export interface Session {
  id?: number;
  sessionId: string;
  projectPath: string;
  projectName: string;
  filePath: string;
  rating?: number | null;
  tags: string; // JSON 数组字符串
  isArchived: boolean;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

/**
 * 会话文件信息接口
 */
export interface SessionFileInfo {
  session_id: string;
  file_path: string;
  display_name?: string;
  summary?: string;
  modified_time: string;
  /** 项目路径（所属监控目录路径） */
  projectPath: string;
}

/**
 * 项目分组（包含会话列表）
 */
export interface ProjectGroup {
  projectName: string;
  projectPath: string;
  sessions: Session[];
  sessionCount: number;
}

/**
 * 会话过滤条件
 */
export interface SessionFilters {
  searchQuery: string;
  selectedProject?: string;
  selectedTags: string[];
  minRating?: number;
  showArchived: boolean;
}

/**
 * 设置会话评分请求
 */
export interface SetSessionRatingRequest {
  sessionId: string;
  rating: number | null;
}

/**
 * 设置会话标签请求
 */
export interface SetSessionTagsRequest {
  sessionId: string;
  tags: string[];
}

// ==================== Store State ====================

interface SessionState {
  // 数据状态
  sessions: Session[];
  projects: ProjectGroup[];
  monitoredDirectories: MonitoredDirectory[];
  loading: boolean;
  error: string | null;
  filters: SessionFilters;
  customDirectory: string | null;

  // Actions
  scanSessions: () => Promise<void>;
  scanDirectory: (directory: string) => Promise<void>;
  setActiveSessions: () => Promise<void>;
  setArchivedSessions: () => Promise<void>;
  setSessionRating: (request: SetSessionRatingRequest) => Promise<void>;
  setSessionTags: (request: SetSessionTagsRequest) => Promise<void>;
  archiveSession: (sessionId: string) => Promise<void>;
  unarchiveSession: (sessionId: string) => Promise<void>;
  updateFilters: (filters: Partial<SessionFilters>) => void;
  resetFilters: () => void;
  clearError: () => void;

  // 监控目录管理操作
  fetchMonitoredDirectories: () => Promise<void>;
  addMonitoredDirectory: (path: string, name: string) => Promise<void>;
  removeMonitoredDirectory: (id: number) => Promise<void>;
  toggleMonitoredDirectory: (id: number) => Promise<void>;
  updateMonitoredDirectory: (directory: MonitoredDirectory) => Promise<void>;

  // 计算属性
  getFilteredSessions: () => Session[];
  getProjectGroups: () => ProjectGroup[];
}

// ==================== 辅助函数 ====================

/**
 * 解析标签 JSON 字符串
 */
function parseTags(tagsJson: string): string[] {
  if (!tagsJson || tagsJson === '[]') {
    return [];
  }
  try {
    return JSON.parse(tagsJson) as string[];
  } catch {
    return [];
  }
}

/**
 * 按项目分组会话
 */
function groupSessionsByProject(sessions: Session[]): ProjectGroup[] {
  const projectMap = new Map<string, ProjectGroup>();

  sessions.forEach((session) => {
    const key = session.projectPath;

    if (!projectMap.has(key)) {
      projectMap.set(key, {
        projectName: session.projectName,
        projectPath: session.projectPath,
        sessions: [],
        sessionCount: 0,
      });
    }

    const group = projectMap.get(key)!;
    group.sessions.push(session);
    group.sessionCount++;
  });

  // 转换为数组并按会话数量排序
  return Array.from(projectMap.values()).sort(
    (a, b) => b.sessionCount - a.sessionCount
  );
}

/**
 * 过滤会话
 */
function filterSessions(
  sessions: Session[],
  filters: SessionFilters
): Session[] {
  return sessions.filter((session) => {
    // 归档过滤
    if (!filters.showArchived && session.isArchived) {
      return false;
    }

    // 项目过滤
    if (
      filters.selectedProject &&
      session.projectPath !== filters.selectedProject
    ) {
      return false;
    }

    // 评分过滤
    if (
      filters.minRating !== undefined &&
      (session.rating === null || session.rating === undefined || session.rating < filters.minRating)
    ) {
      return false;
    }

    // 标签过滤
    if (filters.selectedTags.length > 0) {
      const sessionTags = parseTags(session.tags);
      const hasAllTags = filters.selectedTags.every((tag) =>
        sessionTags.includes(tag)
      );
      if (!hasAllTags) {
        return false;
      }
    }

    // 搜索过滤（搜索会话 ID 和项目名称）
    if (filters.searchQuery) {
      const query = filters.searchQuery.toLowerCase();
      const matchesSearch =
        session.sessionId.toLowerCase().includes(query) ||
        session.projectName.toLowerCase().includes(query) ||
        session.projectPath.toLowerCase().includes(query);

      if (!matchesSearch) {
        return false;
      }
    }

    return true;
  });
}

// ==================== Store 实现 ====================

export const useSessionStore = create<SessionState>()(
  immer((set, get) => ({
    // 初始状态
    sessions: [],
    projects: [],
    monitoredDirectories: [],
    loading: false,
    error: null,
    customDirectory: null,
    filters: {
      searchQuery: '',
      selectedProject: undefined,
      selectedTags: [],
      showArchived: false,
    },

    // 扫描会话（默认目录）
    scanSessions: async () => {
      debugLog('scanSessions', 'start');
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // 调用后端扫描命令（返回 SessionMeta[]，包含基本字段）
        const result = await invoke<
          Array<{
            sessionId: string;
            projectPath: string;
            projectName: string;
            createdAt: string;
            updatedAt: string;
            messageCount: number;
            isActive: boolean;
          }>
        >('scan_sessions');

        debugLog('scanSessions', 'success', result);

        // 转换为 Session 类型（缺少详细字段，需要从数据库获取完整信息）
        // 这里暂时使用扫描结果的基本信息
        const sessions: Session[] = result.map((meta) => ({
          sessionId: meta.sessionId,
          projectPath: meta.projectPath,
          projectName: meta.projectName,
          filePath: '', // 需要从数据库获取
          isActive: meta.isActive,
          createdAt: meta.createdAt,
          updatedAt: meta.updatedAt,
          tags: '[]',
          isArchived: false,
        }));

        set((state) => {
          state.sessions = sessions;
          state.projects = groupSessionsByProject(sessions);
          state.loading = false;
        });
      } catch (error) {
        debugLog('scanSessions', 'error', error);
        set((state) => {
          state.error = `扫描会话失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 扫描指定目录
    scanDirectory: async (directory: string) => {
      debugLog('scanDirectory', 'start', directory);
      set((state) => {
        state.loading = true;
        state.error = null;
        state.customDirectory = directory;
      });

      try {
        const result = await invoke<
          Array<{
            sessionId: string;
            projectPath: string;
            projectName: string;
            createdAt: string;
            updatedAt: string;
            messageCount: number;
            isActive: boolean;
          }>
        >('scan_directory', { directory });

        debugLog('scanDirectory', 'success', result.length);

        const sessions: Session[] = result.map((meta) => ({
          sessionId: meta.sessionId,
          projectPath: meta.projectPath,
          projectName: meta.projectName,
          filePath: '',
          isActive: meta.isActive,
          createdAt: meta.createdAt,
          updatedAt: meta.updatedAt,
          tags: '[]',
          isArchived: false,
        }));

        set((state) => {
          state.sessions = sessions;
          state.projects = groupSessionsByProject(sessions);
          state.loading = false;
        });
      } catch (error) {
        debugLog('scanDirectory', 'error', error);
        set((state) => {
          state.error = `扫描目录失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 加载会话列表（使用 scan_sessions 命令）
    setActiveSessions: async () => {
      debugLog('setActiveSessions', 'start - using scan_sessions');

      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        // 调用 scan_sessions 命令获取会话列表
        const result = await invoke<
          Array<{
            sessionId: string;
            projectPath: string;
            projectName: string;
            createdAt: string;
            updatedAt: string;
            messageCount: number;
            isActive: boolean;
          }>
        >('scan_sessions');

        debugLog('setActiveSessions', 'scan_sessions success', result.length);

        // 转换为 Session 类型
        const sessions: Session[] = result.map((meta) => ({
          sessionId: meta.sessionId,
          projectPath: meta.projectPath,
          projectName: meta.projectName,
          filePath: '',
          isActive: meta.isActive,
          createdAt: meta.createdAt,
          updatedAt: meta.updatedAt,
          tags: '[]',
          isArchived: false,
        }));

        set((state) => {
          state.sessions = sessions;
          state.projects = groupSessionsByProject(sessions);
          state.loading = false;
        });
      } catch (error) {
        debugLog('setActiveSessions', 'error', error);
        set((state) => {
          state.error = `加载会话失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 获取归档会话
    setArchivedSessions: async () => {
      debugLog('setArchivedSessions', 'start');
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        const sessions = await invoke<Session[]>('get_archived_sessions');
        debugLog('setArchivedSessions', 'success', sessions.length);

        set((state) => {
          state.sessions = sessions;
          state.projects = groupSessionsByProject(sessions);
          state.loading = false;
        });
      } catch (error) {
        debugLog('setArchivedSessions', 'error', error);
        set((state) => {
          state.error = `获取归档会话失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 设置会话评分
    setSessionRating: async (request) => {
      debugLog('setSessionRating', request);
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('set_session_rating', {
          request: {
            sessionId: request.sessionId,
            rating: request.rating,
          },
        });

        // 更新本地状态
        set((state) => {
          const session = state.sessions.find(
            (s) => s.sessionId === request.sessionId
          );
          if (session) {
            session.rating = request.rating ?? undefined;
          }
          state.loading = false;
        });
      } catch (error) {
        debugLog('setSessionRating', 'error', error);
        set((state) => {
          state.error = `设置评分失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 设置会话标签
    setSessionTags: async (request) => {
      debugLog('setSessionTags', request);
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('set_session_tags', {
          request: {
            sessionId: request.sessionId,
            tags: request.tags,
          },
        });

        // 更新本地状态
        set((state) => {
          const session = state.sessions.find(
            (s) => s.sessionId === request.sessionId
          );
          if (session) {
            session.tags = JSON.stringify(request.tags);
          }
          state.loading = false;
        });
      } catch (error) {
        debugLog('setSessionTags', 'error', error);
        set((state) => {
          state.error = `设置标签失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 归档会话
    archiveSession: async (sessionId) => {
      debugLog('archiveSession', sessionId);
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('archive_session', { sessionId });

        // 更新本地状态
        set((state) => {
          const session = state.sessions.find((s) => s.sessionId === sessionId);
          if (session) {
            session.isArchived = true;
          }
          state.loading = false;
        });
      } catch (error) {
        debugLog('archiveSession', 'error', error);
        set((state) => {
          state.error = `归档会话失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 取消归档会话
    unarchiveSession: async (sessionId) => {
      debugLog('unarchiveSession', sessionId);
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('unarchive_session', { sessionId });

        // 更新本地状态
        set((state) => {
          const session = state.sessions.find((s) => s.sessionId === sessionId);
          if (session) {
            session.isArchived = false;
          }
          state.loading = false;
        });
      } catch (error) {
        debugLog('unarchiveSession', 'error', error);
        set((state) => {
          state.error = `取消归档失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 更新过滤条件
    updateFilters: (newFilters) => {
      set((state) => {
        state.filters = { ...state.filters, ...newFilters };
      });
    },

    // 重置过滤条件
    resetFilters: () => {
      set((state) => {
        state.filters = {
          searchQuery: '',
          selectedProject: undefined,
          selectedTags: [],
          showArchived: false,
        };
      });
    },

    // 清除错误
    clearError: () => {
      set((state) => {
        state.error = null;
      });
    },

    // 获取过滤后的会话
    getFilteredSessions: () => {
      const { sessions, filters } = get();
      return filterSessions(sessions, filters);
    },

    // 获取项目分组
    getProjectGroups: () => {
      const { sessions } = get();
      return groupSessionsByProject(sessions);
    },

    // ==================== 监控目录管理操作 ====================

    // 获取所有监控目录
    fetchMonitoredDirectories: async () => {
      debugLog('fetchMonitoredDirectories', 'start');
      set((state) => {
        state.error = null;
      });

      try {
        const directories = await invoke<MonitoredDirectory[]>('get_monitored_directories');
        debugLog('fetchMonitoredDirectories', 'success', directories.length);

        set((state) => {
          state.monitoredDirectories = directories;
        });
      } catch (error) {
        debugLog('fetchMonitoredDirectories', 'error', error);
        set((state) => {
          state.error = `获取监控目录失败: ${parseError(error)}`;
        });
        throw error;
      }
    },

    // 添加监控目录
    addMonitoredDirectory: async (path: string, name: string) => {
      debugLog('addMonitoredDirectory', 'start', { path, name });
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        const newDirectory = await invoke<MonitoredDirectory>('add_monitored_directory', {
          path,
          name,
        });
        debugLog('addMonitoredDirectory', 'success');

        set((state) => {
          state.monitoredDirectories.push(newDirectory);
          state.loading = false;
        });
      } catch (error) {
        debugLog('addMonitoredDirectory', 'error', error);
        set((state) => {
          state.error = `添加监控目录失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 删除监控目录
    removeMonitoredDirectory: async (id: number) => {
      debugLog('removeMonitoredDirectory', 'start', id);
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('remove_monitored_directory', { id });
        debugLog('removeMonitoredDirectory', 'success');

        set((state) => {
          state.monitoredDirectories = state.monitoredDirectories.filter((d) => d.id !== id);
          state.loading = false;
        });
      } catch (error) {
        debugLog('removeMonitoredDirectory', 'error', error);
        set((state) => {
          state.error = `删除监控目录失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },

    // 切换监控目录的启用状态
    toggleMonitoredDirectory: async (id: number) => {
      debugLog('toggleMonitoredDirectory', 'start', id);
      set((state) => {
        state.error = null;
      });

      try {
        const isActive = await invoke<boolean>('toggle_monitored_directory', { id });
        debugLog('toggleMonitoredDirectory', 'success', isActive);

        set((state) => {
          const directory = state.monitoredDirectories.find((d) => d.id === id);
          if (directory) {
            directory.is_active = isActive;
          }
        });
      } catch (error) {
        debugLog('toggleMonitoredDirectory', 'error', error);
        set((state) => {
          state.error = `切换目录状态失败: ${parseError(error)}`;
        });
        throw error;
      }
    },

    // 更新监控目录
    updateMonitoredDirectory: async (directory: MonitoredDirectory) => {
      debugLog('updateMonitoredDirectory', 'start', directory.id);
      set((state) => {
        state.loading = true;
        state.error = null;
      });

      try {
        await invoke('update_monitored_directory', { directory });
        debugLog('updateMonitoredDirectory', 'success');

        set((state) => {
          const index = state.monitoredDirectories.findIndex((d) => d.id === directory.id);
          if (index !== -1) {
            state.monitoredDirectories[index] = directory;
          }
          state.loading = false;
        });
      } catch (error) {
        debugLog('updateMonitoredDirectory', 'error', error);
        set((state) => {
          state.error = `更新监控目录失败: ${parseError(error)}`;
          state.loading = false;
        });
        throw error;
      }
    },
  }))
);

// ==================== 便捷 Hooks ====================

/**
 * 获取所有会话
 */
export const useSessions = () => useSessionStore((state) => state.sessions);

/**
 * 获取过滤后的会话（使用浅比较避免无限循环）
 */
export const useFilteredSessions = () => {
  const sessions = useSessionStore((state) => state.sessions);
  const filters = useSessionStore((state) => state.filters);
  return useMemo(() => filterSessions(sessions, filters), [sessions, filters]);
};

/**
 * 获取项目分组（使用 useMemo 缓存结果）
 */
export const useProjectGroups = () => {
  const sessions = useSessionStore((state) => state.sessions);
  return useMemo(() => groupSessionsByProject(sessions), [sessions]);
};

/**
 * 获取加载状态
 */
export const useSessionsLoading = () => useSessionStore((state) => state.loading);

/**
 * 获取错误信息
 */
export const useSessionsError = () => useSessionStore((state) => state.error);

/**
 * 获取会话操作（稳定引用）
 */
export const useSessionActions = () => {
  return useMemo(() => {
    const store = useSessionStore.getState();
    return {
      scanSessions: store.scanSessions,
      scanDirectory: store.scanDirectory,
      setActiveSessions: store.setActiveSessions,
      setArchivedSessions: store.setArchivedSessions,
      setSessionRating: store.setSessionRating,
      setSessionTags: store.setSessionTags,
      archiveSession: store.archiveSession,
      unarchiveSession: store.unarchiveSession,
      updateFilters: store.updateFilters,
      resetFilters: store.resetFilters,
      clearError: store.clearError,
    };
  }, []);
};

/**
 * 获取监控目录列表
 */
export const useMonitoredDirectories = () => useSessionStore((state) => state.monitoredDirectories);

/**
 * 获取监控目录操作（稳定引用）
 */
export const useMonitoredDirectoryActions = () => {
  return useMemo(() => {
    const store = useSessionStore.getState();
    return {
      fetchMonitoredDirectories: store.fetchMonitoredDirectories,
      addMonitoredDirectory: store.addMonitoredDirectory,
      removeMonitoredDirectory: store.removeMonitoredDirectory,
      toggleMonitoredDirectory: store.toggleMonitoredDirectory,
      updateMonitoredDirectory: store.updateMonitoredDirectory,
    };
  }, []);
};
