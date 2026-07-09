import * as React from 'react'
import { Slot } from 'radix-ui'

import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'

type OverflowTooltipProps = {
  asChild?: boolean
  children: React.ReactNode
  content?: React.ReactNode
  disabled?: boolean
  onFocus?: React.FocusEventHandler<HTMLElement>
  onPointerEnter?: React.PointerEventHandler<HTMLElement>
  side?: React.ComponentProps<typeof TooltipContent>['side']
  sideOffset?: React.ComponentProps<typeof TooltipContent>['sideOffset']
  tooltipClassName?: string
}

export function OverflowTooltip({
  asChild = false,
  children,
  content = children,
  disabled = false,
  onFocus,
  onPointerEnter,
  side = 'top',
  sideOffset,
  tooltipClassName
}: OverflowTooltipProps) {
  const triggerRef = React.useRef<HTMLElement | null>(null)
  const [open, setOpen] = React.useState(false)
  const [isOverflowing, setIsOverflowing] = React.useState(false)

  const measureOverflow = React.useCallback(() => {
    const element = triggerRef.current
    const nextIsOverflowing =
      !disabled &&
      element != null &&
      (element.scrollWidth > element.clientWidth + 1 ||
        element.scrollHeight > element.clientHeight + 1)

    setIsOverflowing(nextIsOverflowing)
    return nextIsOverflowing
  }, [disabled])

  const handleOpenChange = React.useCallback(
    (nextOpen: boolean) => {
      if (!nextOpen) {
        setOpen(false)
        return
      }

      setOpen(measureOverflow())
    },
    [measureOverflow]
  )

  function handlePointerEnter(event: React.PointerEvent<HTMLElement>) {
    onPointerEnter?.(event)
    measureOverflow()
  }

  function handleFocus(event: React.FocusEvent<HTMLElement>) {
    onFocus?.(event)
    measureOverflow()
  }

  const triggerProps = {
    ref: triggerRef,
    'data-slot': 'overflow-tooltip-trigger',
    onFocus: handleFocus,
    onPointerEnter: handlePointerEnter
  }
  const trigger = asChild ? (
    <Slot.Root {...triggerProps}>{children}</Slot.Root>
  ) : (
    <span {...triggerProps} className="truncate">
      {children}
    </span>
  )

  return (
    <Tooltip open={open} onOpenChange={handleOpenChange}>
      <TooltipTrigger asChild>{trigger}</TooltipTrigger>
      {isOverflowing ? (
        <TooltipContent side={side} sideOffset={sideOffset} className={tooltipClassName}>
          {content}
        </TooltipContent>
      ) : null}
    </Tooltip>
  )
}
