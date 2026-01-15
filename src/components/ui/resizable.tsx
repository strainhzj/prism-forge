/**
 * Resizable 组件
 * 基于 react-resizable-panels v4.x 的封装
 * 参考：https://ui.shadcn.com/docs/components/resizable
 */

import { cn } from "@/lib/utils"
import {
  Group as ResizablePrimitiveGroup,
  Panel as ResizablePrimitivePanel,
  Separator as ResizablePrimitiveSeparator,
  type GroupProps,
  type PanelProps,
  type SeparatorProps,
} from "react-resizable-panels"

const ResizablePanelGroup = ({
  className,
  ...props
}: GroupProps) => (
  <ResizablePrimitiveGroup
    className={cn("flex h-full w-full", className)}
    {...props}
  />
)

const ResizablePanel = ({
  className,
  ...props
}: PanelProps) => (
  <ResizablePrimitivePanel
    className={cn("flex flex-col overflow-hidden", className)}
    {...props}
  />
)

const ResizableHandle = ({
  className,
  withHandle = false,
  ...props
}: SeparatorProps & { withHandle?: boolean }) => (
  <ResizablePrimitiveSeparator
    className={cn(
      "relative flex w-px bg-border transition-all hover:bg-accent",
      withHandle && "data-[handle]:w-2 data-[handle]:bg-accent",
      className
    )}
    {...props}
  />
)

export { ResizablePanelGroup, ResizablePanel, ResizableHandle }
