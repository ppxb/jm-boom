import type { ComponentProps } from 'react'

import { DrawerContent } from '@/components/ui/drawer'
import { cn } from 'cn'

export function SideDrawerContent({ className, ...props }: ComponentProps<typeof DrawerContent>) {
  return (
    <DrawerContent
      className={cn(
        'h-full w-[82vw] max-w-[320px] overflow-hidden rounded-l-2xl rounded-r-none p-0 [--drawer-inset:0px] sm:w-[420px] sm:max-w-[420px]',
        className
      )}
      {...props}
    />
  )
}
