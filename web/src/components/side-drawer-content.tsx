import type { ComponentProps } from 'react'

import { DrawerContent } from '@/components/ui/drawer'
import { cn } from '@/lib/utils'

export function SideDrawerContent({ className, ...props }: ComponentProps<typeof DrawerContent>) {
  return (
    <DrawerContent
      className={cn(
        'h-full w-[82vw] max-w-[320px] overflow-hidden rounded-l-2xl p-0 before:inset-0 before:rounded-l-2xl before:rounded-r-none',
        'data-[vaul-drawer-direction=right]:w-[82vw] data-[vaul-drawer-direction=right]:max-w-[320px]',
        'data-[vaul-drawer-direction=right]:sm:w-[420px] data-[vaul-drawer-direction=right]:sm:max-w-[420px]',
        className
      )}
      {...props}
    />
  )
}
