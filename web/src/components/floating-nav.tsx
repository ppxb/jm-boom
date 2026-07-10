import { Link } from '@tanstack/react-router'
import type { LucideIcon } from 'lucide-react'
import { Fragment, type MouseEvent } from 'react'

import { buttonVariants } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import type { FileRoutesByTo } from '@/routeTree.gen'

type FloatingNavTo = keyof FileRoutesByTo

export type FloatingNavItem = {
  id: string
  icon: LucideIcon
  label: string
  to: FloatingNavTo
  separatorBefore?: boolean
}

type FloatingNavProps = {
  items: FloatingNavItem[]
  activeId: string | undefined
  onItemClick: (item: FloatingNavItem, event: MouseEvent<HTMLAnchorElement>) => void
}

export function FloatingNav({ items, activeId, onItemClick }: FloatingNavProps) {
  if (items.length === 0) return null

  return (
    <nav className="fixed top-1/2 left-6 z-50 -translate-y-1/2 rounded-full border border-border/70 p-1 backdrop-blur">
      <ul className="flex flex-col items-center gap-1">
        {items.map(item => (
          <Fragment key={item.id}>
            {item.separatorBefore ? (
              <li aria-hidden="true" className="my-1 h-px w-6 bg-border/70" />
            ) : null}
            <NavItem item={item} isActive={item.id === activeId} onItemClick={onItemClick} />
          </Fragment>
        ))}
      </ul>
    </nav>
  )
}

type NavItemProps = {
  item: FloatingNavItem
  isActive: boolean
  onItemClick: (item: FloatingNavItem, event: MouseEvent<HTMLAnchorElement>) => void
}

function NavItem({ item, isActive, onItemClick }: NavItemProps) {
  function handleClick(event: MouseEvent<HTMLAnchorElement>) {
    onItemClick(item, event)
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Link
          to={item.to}
          onClick={handleClick}
          className={buttonVariants({ variant: isActive ? 'default' : 'ghost', size: 'icon' })}
        >
          <item.icon className="size-4" />
        </Link>
      </TooltipTrigger>
      <TooltipContent side="right">{item.label}</TooltipContent>
    </Tooltip>
  )
}
