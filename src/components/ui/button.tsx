import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

/**
 * Button 组件变体配置
 *
 * 使用 CVA (class-variance-authority) 管理按钮样式变体
 */
const buttonVariants = cva(
  // 基础样式：所有变体共享
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        primary:
          "bg-primary text-primary-foreground shadow hover:bg-primary/90",
        secondary:
          "bg-secondary text-secondary-foreground shadow-sm hover:bg-secondary/80",
        destructive:
          "bg-destructive text-destructive-foreground shadow-sm hover:bg-destructive/90",
        outline:
          "border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground",
        ghost:
          "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 rounded-md px-3 text-xs",
        lg: "h-10 rounded-md px-8",
        icon: "h-9 w-9",
      },
    },
    defaultVariants: {
      variant: "primary",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

/**
 * Button 组件
 *
 * 支持多种样式变体和尺寸的可复用按钮组件
 *
 * @example
 * // 主要按钮
 * <Button>点击我</Button>
 *
 * // 次要按钮
 * <Button variant="secondary">取消</Button>
 *
 * // 危险按钮
 * <Button variant="destructive">删除</Button>
 *
 * // 图标按钮
 * <Button size="icon"><RefreshIcon /></Button>
 */
const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, ...props }, ref) => {
    return (
      <button
        className={cn(buttonVariants({ variant, size, className }))}
        style={{
          ...(variant === 'primary' && {
            backgroundColor: 'var(--color-accent-warm)',
            color: '#FFFFFF',
          }),
          ...(variant === 'outline' && {
            borderColor: 'var(--color-border-light)',
            backgroundColor: 'var(--color-bg-card)',
            color: 'var(--color-text-primary)',
          }),
        }}
        ref={ref}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }
