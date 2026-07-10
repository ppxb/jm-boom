import { createRootRoute, Outlet, useRouter } from '@tanstack/react-router'
import { ArrowLeftIcon, CircleAlertIcon, HomeIcon } from 'lucide-react'
import { useState } from 'react'

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogTitle
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useSettingsStore } from '@/stores/settings-store'

export const Route = createRootRoute({
  component: RootLayout,
  notFoundComponent: NotFoundComponent
})

function RootLayout() {
  return (
    <>
      <Outlet />
      <NsfwStartupDialog />
    </>
  )
}

function NsfwStartupDialog() {
  const nsfwWarningDismissed = useSettingsStore(state => state.nsfwWarningDismissed)
  const dismissNsfwWarning = useSettingsStore(state => state.dismissNsfwWarning)
  const [open, setOpen] = useState(!nsfwWarningDismissed)

  if (nsfwWarningDismissed) {
    return null
  }

  function handleDismissPermanently() {
    dismissNsfwWarning()
    setOpen(false)
  }

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogContent>
        <div className="flex items-start gap-3 py-1">
          <div className="flex size-10 shrink-0 items-center justify-center rounded-full bg-destructive/10 dark:bg-destructive/10">
            <CircleAlertIcon className="size-5 text-destructive" />
          </div>
          <div className="flex flex-col justify-center gap-1">
            <AlertDialogTitle className="text-sm font-semibold">NSFW 内容警告</AlertDialogTitle>
            <AlertDialogDescription className="text-sm text-muted-foreground">
              本应用可能展示不适合未成年人或公共场合浏览的成人向内容。请确认你已达到当地法定年龄，并在私密、安全的环境中使用。
            </AlertDialogDescription>
          </div>
        </div>
        <AlertDialogFooter>
          <AlertDialogAction variant="destructive" onClick={handleDismissPermanently}>
            不再提示
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

function NotFoundComponent() {
  const router = useRouter()

  return (
    <div className="flex min-h-svh flex-col items-center justify-center bg-muted">
      <div className="w-full max-w-md">
        <Card>
          <CardContent className="space-y-6">
            <div className="flex flex-col items-center gap-6 text-center">
              <div className="font-mono text-8xl font-bold text-primary">404</div>
              <h1 className="text-2xl font-bold">页面未找到</h1>
              <p className="text-sm text-muted-foreground">抱歉，你访问的页面不存在或已被删除</p>
            </div>

            <div className="flex gap-2">
              <Button variant="outline" className="flex-1" onClick={() => router.history.back()}>
                <ArrowLeftIcon className="size-4" />
                返回
              </Button>
              <Button className="flex-1" onClick={() => router.navigate({ to: '/' })}>
                <HomeIcon className="size-4" />
                回到首页
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
