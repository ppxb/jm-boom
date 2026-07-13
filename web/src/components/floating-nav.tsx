import { Link } from '@tanstack/react-router'
import type { LucideIcon } from 'lucide-react'

import { buttonVariants } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import type { FileRoutesByTo } from '@/routeTree.gen'

type FloatingNavTo = keyof FileRoutesByTo

export type FloatingNavItem = {
  id: string
  icon: LucideIcon
  label: string
  to: FloatingNavTo
}

type FloatingNavProps = {
  items: FloatingNavItem[]
  activeId: string | undefined
}

export function FloatingNav({ items, activeId }: FloatingNavProps) {
  return (
    <nav className="fixed bottom-4 left-1/2 z-50 -translate-x-1/2 rounded-full border border-border/70 bg-background/85 p-1.5 text-foreground backdrop-blur sm:bottom-4 sm:p-1">
      <ul className="flex items-center gap-1.5 sm:gap-1">
        {items.map(item => (
          <NavItem key={item.id} item={item} isActive={item.id === activeId} />
        ))}
      </ul>
    </nav>
  )
}

type NavItemProps = {
  item: FloatingNavItem
  isActive: boolean
}

function NavItem({ item, isActive }: NavItemProps) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Link
          to={item.to}
          className={`${buttonVariants({ variant: isActive ? 'default' : 'ghost', size: 'icon' })} size-11 sm:size-9`}
          aria-label={item.label}
        >
          <item.icon className="size-5" />
        </Link>
      </TooltipTrigger>
      <TooltipContent side="top">{item.label}</TooltipContent>
    </Tooltip>
  )
}
