import { createFileRoute, Outlet, useRouterState } from '@tanstack/react-router'
import { BookmarkIcon, CompassIcon, DownloadIcon, SettingsIcon, ShellIcon } from 'lucide-react'

import { FloatingNav, type FloatingNavItem } from '@/components/floating-nav'

export const Route = createFileRoute('/_app')({
  component: AppRoute
})

const NAV_ITEMS: FloatingNavItem[] = [
  { id: 'bookshelf', icon: ShellIcon, label: '书架', to: '/bookshelf' },
  { id: 'explore', icon: CompassIcon, label: '探索', to: '/explore' },
  { id: 'favorites', icon: BookmarkIcon, label: '收藏', to: '/favorites' },
  { id: 'downloads', icon: DownloadIcon, label: '下载', to: '/downloads' },
  { id: 'settings', icon: SettingsIcon, label: '设置', to: '/settings' }
]

function AppRoute() {
  const pathname = useRouterState({
    select: state => state.location.pathname
  })
  const activeId = NAV_ITEMS.find(item => pathname.startsWith(item.to))?.id
  const showNav = !pathname.startsWith('/comic/')

  return (
    <div className="relative min-h-dvh">
      {showNav ? <FloatingNav items={NAV_ITEMS} activeId={activeId} /> : null}
      <Outlet />
    </div>
  )
}
