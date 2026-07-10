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
  if (items.length === 0) return null

  return (
    <nav className="fixed bottom-[calc(env(safe-area-inset-bottom)+1rem)] left-1/2 z-50 -translate-x-1/2 rounded-full border border-border/70 p-1 backdrop-blur">
      <ul className="flex items-center gap-1">
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
          className={buttonVariants({ variant: isActive ? 'default' : 'ghost', size: 'icon' })}
          aria-label={item.label}
        >
          <item.icon className="size-4" />
        </Link>
      </TooltipTrigger>
      <TooltipContent side="top">{item.label}</TooltipContent>
    </Tooltip>
  )
}
