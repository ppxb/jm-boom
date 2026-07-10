import { createFileRoute, Outlet, useNavigate, useRouterState } from '@tanstack/react-router'
import {
  BookmarkIcon,
  CalendarDaysIcon,
  DownloadIcon,
  HistoryIcon,
  HouseIcon,
  SearchIcon,
  TrophyIcon,
  SettingsIcon,
  UserRoundIcon
} from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { toast } from 'sonner'

import { FloatingNav, type FloatingNavItem } from '@/components/floating-nav'
import { LoginDialog } from '@/features/user/login-dialog'
import { useUserStore } from '@/stores/user-store'

export const Route = createFileRoute('/_app')({
  component: AppRoute
})

const NAV_ITEMS: FloatingNavItem[] = [
  { id: 'home', icon: HouseIcon, label: '首页', to: '/' },
  { id: 'weekly', icon: CalendarDaysIcon, label: '每周推荐', to: '/weekly' },
  { id: 'ranking', icon: TrophyIcon, label: '排行榜', to: '/ranking' },
  { id: 'favorites', icon: BookmarkIcon, label: '收藏', to: '/favorites' },
  { id: 'history', icon: HistoryIcon, label: '历史观看', to: '/history' },
  { id: 'downloads', icon: DownloadIcon, label: '下载', to: '/downloads' },
  { id: 'settings', icon: SettingsIcon, label: '设置', to: '/settings' },
  { id: 'me', icon: UserRoundIcon, label: '我的', to: '/me' },
  { id: 'search', icon: SearchIcon, label: '搜索', to: '/search', separatorBefore: true }
]

function AppRoute() {
  const navigate = useNavigate()
  const user = useUserStore(state => state.user)
  const initializeUser = useUserStore(state => state.initialize)
  const hasInitializedUserRef = useRef(false)

  const [isLoginOpen, setIsLoginOpen] = useState(false)

  const pathname = useRouterState({
    select: state => state.location.pathname
  })
  const navItems = user ? NAV_ITEMS : NAV_ITEMS.filter(item => item.id !== 'favorites')

  const activeId = [...navItems]
    .reverse()
    .find(item => (item.to === '/' ? pathname === '/' : pathname.startsWith(item.to)))?.id

  useEffect(() => {
    if (hasInitializedUserRef.current) {
      return
    }

    hasInitializedUserRef.current = true
    initializeUser().catch(error => {
      console.error('Failed to restore user session', error)
      toast.error('自动登录失败', {
        description: error instanceof Error ? error.message : String(error)
      })
    })
  }, [initializeUser])

  return (
    <div className="relative min-h-screen">
      <FloatingNav
        items={navItems}
        activeId={activeId}
        onItemClick={(item, event) => {
          if (item.id !== 'me' || user) {
            return
          }
          event.preventDefault()
          setIsLoginOpen(true)
        }}
      />
      <LoginDialog
        open={isLoginOpen}
        onOpenChange={setIsLoginOpen}
        onLoginSuccess={() => navigate({ to: '/me' })}
      />
      <Outlet />
    </div>
  )
}
