import * as React from "react"
import { cn } from "@/lib/utils"

export interface LabelProps
  extends React.LabelHTMLAttributes<HTMLLabelElement> {}

/**
 * Label 组件
 *
 * 表单标签组件，用于与输入框关联
 *
 * @example
 * <div>
 *   <Label htmlFor="email">邮箱</Label>
 *   <Input id="email" type="email" />
 * </div>
 */
const Label = React.forwardRef<HTMLLabelElement, LabelProps>(
  ({ className, ...props }, ref) => (
    <label
      ref={ref}
      className={cn(
        "text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70",
        className
      )}
      style={{ color: 'var(--color-text-primary)' }}
      {...props}
    />
  )
)
Label.displayName = "Label"

export { Label }
