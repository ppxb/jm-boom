import { useQueryClient } from '@tanstack/react-query'
import { useState } from 'react'
import { toast } from 'sonner'
import { LoaderCircleIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { queryKeys } from '@/lib/query-keys'
import { useSettingsStore } from '@/stores/settings-store'
import { useUserStore } from '@/stores/user-store'
import { Field, FieldGroup, FieldLabel } from '@/components/ui/field'

type LoginDialogProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  onLoginSuccess?: () => void
}

function formatLoginError(error: unknown) {
  const rawMessage = error instanceof Error ? error.message : String(error)
  const message = rawMessage
    .replace(/\\\//g, '/')
    .replace(/^(Api|Http|Network|Payload|Decode|Decrypt|MissingData|Error):\s*/i, '')
    .replace(/^https?:\/\/[^:\s]+\/login:\s*/i, '')
    .trim()

  if (
    /401|unauthorized|無效的用戶名|无效的用户名|用戶名.*密碼|用户名.*密码|password|credential/i.test(
      message
    )
  ) {
    return '账号或密码错误'
  }

  return message || '请稍后重试'
}

export function LoginDialog({ open, onOpenChange, onLoginSuccess }: LoginDialogProps) {
  const queryClient = useQueryClient()
  const login = useUserStore(state => state.login)
  const isLoggingIn = useUserStore(state => state.isLoggingIn)
  const endpoint = useSettingsStore(state => state.api)
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [rememberLogin, setRememberLogin] = useState(false)

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setUsername('')
      setPassword('')
      setRememberLogin(false)
    }

    onOpenChange(nextOpen)
  }

  async function handleSubmit() {
    const nextUsername = username.trim()
    const nextPassword = password

    if (!nextUsername || !nextPassword.trim()) {
      return
    }

    try {
      await login({ username: nextUsername, password: nextPassword, endpoint, rememberLogin })
      if (rememberLogin) {
        queryClient.setQueryData(queryKeys.savedLoginConfig(), {
          endpoint,
          username: nextUsername,
          autoLogin: true,
          hasPassword: true
        })
      }
      toast.success('登录成功')
      handleOpenChange(false)
      onLoginSuccess?.()
    } catch (error) {
      toast.error('登录失败', {
        description: formatLoginError(error)
      })
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>账号登录</DialogTitle>
          <DialogDescription>使用禁漫天堂账号登录后同步个人资料和签到记录</DialogDescription>
        </DialogHeader>
        <FieldGroup>
          <Field>
            <FieldLabel htmlFor="username">账号或邮箱</FieldLabel>
            <Input
              id="username"
              value={username}
              onChange={event => setUsername(event.target.value)}
              autoFocus
              autoComplete="username"
              placeholder="请输入账号或邮箱"
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="password">密码</FieldLabel>
            <Input
              id="password"
              value={password}
              onChange={event => setPassword(event.target.value)}
              autoComplete="current-password"
              placeholder="请输入密码"
              type="password"
            />
          </Field>
          <label className="flex items-center gap-2 text-sm text-muted-foreground">
            <Checkbox
              checked={rememberLogin}
              onCheckedChange={checked => setRememberLogin(checked === true)}
            />
            自动登录
          </label>
        </FieldGroup>
        <DialogFooter>
          <Button
            onClick={handleSubmit}
            disabled={isLoggingIn || !username.trim() || !password.trim()}
          >
            {isLoggingIn ? <LoaderCircleIcon className="size-4 animate-spin" /> : null}
            {isLoggingIn ? '登录中' : '登录'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
