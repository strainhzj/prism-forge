import { QueryClient } from '@tanstack/react-query'

/**
 * 创建 Tanstack Query 客户端
 *
 * 配置说明：
 * - defaultOptions.staleTime: 数据在 5 分钟内视为新鲜，不会自动重新获取
 * - defaultOptions.gcTime: 未使用的数据在 10 分钟后从内存中清除
 * - defaultOptions.retry: 失败时重试 1 次
 * - defaultOptions.refetchOnWindowFocus: 窗口获得焦点时不自动重新获取（桌面应用不需要）
 */
export function makeQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 5 * 60 * 1000, // 5 分钟
        gcTime: 10 * 60 * 1000, // 10 分钟
        retry: 1,
        refetchOnWindowFocus: false,
      },
    },
  })
}

let browserQueryClient: QueryClient | undefined = undefined

/**
 * 获取浏览器端 QueryClient 单例
 *
 * 在开发环境中使用严格模式，确保每个会话只创建一个客户端
 */
export function getQueryClient() {
  if (import.meta.env.DEV && typeof window !== 'undefined') {
    // 开发环境：确保只有一个客户端实例
    if (!browserQueryClient) {
      browserQueryClient = makeQueryClient()
    }
    return browserQueryClient
  } else {
    // 生产环境：每次都创建新客户端
    return makeQueryClient()
  }
}
