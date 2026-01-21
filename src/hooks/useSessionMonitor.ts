/**
 * useSessionMonitor Hook
 *
 * 监听 Tauri 事件，检测会话文件变更并触发刷新
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useSessionActions } from '@/stores/useSessionStore';

/**
 * 会话变更事件类型
 */
export type SessionChangeKind = 'created' | 'modified' | 'deleted';

/**
 * 会话变更事件
 */
export interface SessionChangeEvent {
  /** 事件类型 */
  kind: SessionChangeKind;
  /** 文件路径 */
  path: string;
  /** 是否为 JSONL 文件 */
  isJsonl: boolean;
  /** 事件时间戳 */
  timestamp: string;
}

/**
 * useSessionMonitor Hook 配置
 */
export interface SessionMonitorOptions {
  /** 是否启用监控 */
  enabled?: boolean;
  /** 防抖延迟（毫秒），默认 2000ms */
  debounceMs?: number;
  /** 变更事件回调 */
  onChange?: (event: SessionChangeEvent) => void;
  /** 刷新回调（防抖后执行） */
  onRefresh?: () => void | Promise<void>;
}

/**
 * useSessionMonitor Hook
 *
 * 监听会话文件变更事件，支持防抖和自动刷新
 *
 * @example
 * const { isRefreshing, lastEvent } = useSessionMonitor({
 *   debounceMs: 2000,
 *   onRefresh: () => refetchSessions(),
 * });
 */
export function useSessionMonitor(options: SessionMonitorOptions = {}) {
  const {
    enabled = true,
    debounceMs = 2000,
    onChange,
    onRefresh,
  } = options;

  // 状态
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [lastEvent, setLastEvent] = useState<SessionChangeEvent | null>(null);
  const [pendingChanges, setPendingChanges] = useState(0);

  // Refs
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const refreshResetTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastEventIdRef = useRef<string>('');
  const onChangeRef = useRef<typeof onChange>(onChange);
  const onRefreshRef = useRef<typeof onRefresh>(onRefresh);
  const debounceMsRef = useRef(debounceMs);
  const isMountedRef = useRef(true);

  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);

  useEffect(() => {
    onRefreshRef.current = onRefresh;
  }, [onRefresh]);

  useEffect(() => {
    debounceMsRef.current = debounceMs;
  }, [debounceMs]);

  useEffect(() => {
    isMountedRef.current = true;
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  /**
   * 触发刷新
   */
  const triggerRefresh = useCallback(async () => {
    const refresh = onRefreshRef.current;
    if (!refresh) return;
    if (!isMountedRef.current) return;

    setIsRefreshing(true);
    try {
      await refresh();
    } catch (error) {
      console.error('刷新失败:', error);
    } finally {
      if (refreshResetTimerRef.current) {
        clearTimeout(refreshResetTimerRef.current);
      }
      refreshResetTimerRef.current = setTimeout(() => {
        if (!isMountedRef.current) return;
        setIsRefreshing(false);
        setPendingChanges(0);
      }, 500);
    }
  }, []);

  /**
   * 处理会话变更事件
   */
  const handleSessionChange = useCallback(
    (event: SessionChangeEvent) => {
      // 事件去重（基于路径+类型+时间戳）
      const eventId = `${event.kind}-${event.path}-${event.timestamp}`;
      if (eventId === lastEventIdRef.current) {
        return; // 重复事件，忽略
      }
      lastEventIdRef.current = eventId;

      console.log('会话文件变更:', event);

      // 更新状态
      setLastEvent(event);
      setPendingChanges((prev) => prev + 1);

      // 调用回调
      onChangeRef.current?.(event);

      // 防抖刷新
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }

      debounceTimerRef.current = setTimeout(() => {
        triggerRefresh();
      }, debounceMsRef.current);
    },
    [triggerRefresh]
  );

  /**
   * 设置事件监听
   */
  useEffect(() => {
    if (!enabled) return;

    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        const unlistenFn = await listen<SessionChangeEvent>(
          'sessions-changed',
          (event) => {
            handleSessionChange(event.payload);
          }
        );
        unlisten = unlistenFn;
        console.log('会话监控已启动');
      } catch (error) {
        console.error('设置会话监控失败:', error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
      }
      if (refreshResetTimerRef.current) {
        clearTimeout(refreshResetTimerRef.current);
      }
    };
  }, [enabled, handleSessionChange]);

  return {
    /** 是否正在刷新 */
    isRefreshing,
    /** 最后一次变更事件 */
    lastEvent,
    /** 待处理的变更数量 */
    pendingChanges,
    /** 手动触发刷新 */
    triggerRefresh,
  };
}

/**
 * 使用简化的会话监控 Hook
 *
 * 自动与 useSessionStore 集成
 */
export function useAutoRefresh() {
  const [lastChange, setLastChange] = useState<SessionChangeEvent | null>(null);
  const { setActiveSessions } = useSessionActions();

  const { pendingChanges, isRefreshing, triggerRefresh } = useSessionMonitor({
    debounceMs: 2000,
    onChange: (event) => {
      setLastChange(event);
    },
    onRefresh: async () => {
      await setActiveSessions();
    },
  });

  return {
    isRefreshing,
    lastChange,
    pendingChanges,
    triggerRefresh,
  };
}
