import { createFileRoute, redirect, useNavigate } from '@tanstack/react-router'
import { BadgeCheckIcon, LogOutIcon } from 'lucide-react'
import { useEffect, useRef } from 'react'
import { toast } from 'sonner'

import { ConfirmDialog } from '@/components/confirm-dialog'
import { PageHeader } from '@/components/page-header'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { useMeSignIn } from '@/features/me/use-me-sign-in'
import { useSettingsStore } from '@/stores/settings-store'
import { useUserStore } from '@/stores/user-store'

export const Route = createFileRoute('/_app/me')({
  beforeLoad: () => {
    if (!useUserStore.getState().user) {
      throw redirect({ to: '/' })
    }
  },
  component: MePage
})

function MePage() {
  const navigate = useNavigate()
  const user = useUserStore(state => state.user)
  const endpoint = useSettingsStore(state => state.api)
  const logout = useUserStore(state => state.logout)
  const signInState = useMeSignIn({ user, endpoint })
  const autoSignInKeyRef = useRef<string | null>(null)

  useEffect(() => {
    if (
      !user ||
      !signInState.data ||
      signInState.isLoading ||
      signInState.isFetching ||
      signInState.isSigning ||
      signInState.todayRecord?.signed
    ) {
      return
    }

    const autoSignInKey = `${endpoint ?? ''}:${user.id}:${new Date().toDateString()}`

    if (autoSignInKeyRef.current === autoSignInKey) {
      return
    }

    autoSignInKeyRef.current = autoSignInKey
    signInState.submitSignIn()
  }, [endpoint, signInState, user])

  if (!user) {
    return null
  }

  async function handleLogout() {
    try {
      await logout()
      toast.success('已退出登录')
      void navigate({ to: '/' })
    } catch (error) {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto flex min-h-screen w-full max-w-6xl flex-col p-[32px_32px_16px_96px]">
        <PageHeader title="个人中心" description="展示用户信息">
          <div className="flex items-center gap-3">
            <Button variant="outline" size="sm" disabled>
              <BadgeCheckIcon className="size-4" />
              {signInState.isSigning
                ? '签到中'
                : signInState.todayRecord?.signed
                  ? '已签到'
                  : '未签到'}
            </Button>
            <ConfirmDialog
              trigger={
                <Button variant="outline" size="sm">
                  <LogOutIcon className="size-4" />
                  登出
                </Button>
              }
              icon={<LogOutIcon className="size-5 text-destructive" />}
              title="退出登录"
              description="退出后会清除当前账号会话，并返回首页。"
              confirmText="确认退出"
              variant="destructive"
              onConfirm={() => void handleLogout()}
            />
          </div>
        </PageHeader>

        <div className="flex flex-1 flex-col items-center justify-center py-10">
          <Avatar className="size-32">
            <AvatarImage src={user.avatarUrl} />
            <AvatarFallback>{user.username.slice(0, 2).toUpperCase()}</AvatarFallback>
          </Avatar>
          <div className="mt-4 flex flex-col items-center gap-2">
            <h2 className="truncate text-4xl font-bold">{user.username}</h2>
            <p className="text-sm text-muted-foreground">UID {user.id}</p>
          </div>

          <div className="mt-12 flex gap-16">
            {[
              {
                label: '经验',
                value: `${user.currentLevelExp}/${user.nextLevelExp}`
              },
              {
                label: '等级',
                value: `${user.level}（${user.levelName || '未命名'}）`
              },
              {
                label: '金币',
                value: user.jCoin.toLocaleString('zh-CN')
              },
              {
                label: '收藏',
                value: `${user.currentCollectCount}/${user.maxCollectCount}`
              }
            ].map(item => (
              <div key={item.label} className="flex flex-col items-center gap-2">
                <div className="text-sm text-muted-foreground">{item.label}</div>
                <div className="truncate text-xl font-bold">{item.value}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </main>
  )
}
