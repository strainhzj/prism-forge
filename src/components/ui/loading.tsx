import { cn } from "@/lib/utils"
import { Loader2 } from "lucide-react"

export interface LoadingProps {
  /**
   * 加载大小
   */
  size?: "sm" | "md" | "lg"
  /**
   * 加载文本
   */
  text?: string
  /**
   * 自定义类名
   */
  className?: string
}

/**
 * Loading 组件
 *
 * 用于显示加载状态
 *
 * @example
 * // 默认加载器
 * <Loading />
 *
 * // 带文本的加载器
 * <Loading text="正在加载..." />
 *
 * // 小尺寸加载器
 * <Loading size="sm" />
 */
export function Loading({ size = "md", text, className }: LoadingProps) {
  const sizeClasses = {
    sm: "h-4 w-4",
    md: "h-6 w-6",
    lg: "h-8 w-8",
  }

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <Loader2 className={cn("animate-spin", sizeClasses[size])} />
      {text && <span className="text-sm text-muted-foreground">{text}</span>}
    </div>
  )
}

/**
 * 全屏加载遮罩组件
 */
export interface LoadingOverlayProps extends LoadingProps {
  /**
   * 是否显示遮罩
   */
  show?: boolean
}

/**
 * LoadingOverlay 组件
 *
 * 全屏加载遮罩，通常用于页面加载状态
 *
 * @example
 * <LoadingOverlay show={isLoading} text="处理中..." />
 */
export function LoadingOverlay({ show = true, text, className }: LoadingOverlayProps) {
  if (!show) return null

  return (
    <div className={cn("fixed inset-0 z-50 flex items-center justify-center bg-background/80 backdrop-blur-sm", className)}>
      <div className="flex flex-col items-center gap-4">
        <Loader2 className="h-12 w-12 animate-spin text-primary" />
        {text && <p className="text-sm text-muted-foreground">{text}</p>}
      </div>
    </div>
  )
}
