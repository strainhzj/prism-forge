import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

/**
 * Separator 组件变体配置
 */
const separatorVariants = cva(
  "shrink-0 bg-border",
  {
    variants: {
      orientation: {
        horizontal: "h-[1px] w-full",
        vertical: "h-full w-[1px]",
      },
    },
    defaultVariants: {
      orientation: "horizontal",
    },
  }
)

export interface SeparatorProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof separatorVariants> {
  /**
   * 分隔线方向
   */
  orientation?: "horizontal" | "vertical"
  /**
   * 是否装饰性（对屏幕阅读器隐藏）
   */
  decorative?: boolean
}

/**
 * Separator 组件
 *
 * 用于在内容之间创建视觉分隔的分隔线组件
 *
 * @example
 * // 水平分隔线（默认）
 * <Separator />
 *
 * // 垂直分隔线
 * <Separator orientation="vertical" />
 *
 * // 带样式的分隔线
 * <Separator className="my-4" style={{ backgroundColor: '#333' }} />
 */
const Separator = React.forwardRef<HTMLDivElement, SeparatorProps>(
  (
    { className, orientation = "horizontal", decorative = true, ...props },
    ref
  ) => {
    return (
      <div
        role={decorative ? "none" : "separator"}
        aria-orientation={orientation}
        className={cn(separatorVariants({ orientation }), className)}
        ref={ref}
        {...props}
      />
    )
  }
)
Separator.displayName = "Separator"

export { Separator, separatorVariants }
