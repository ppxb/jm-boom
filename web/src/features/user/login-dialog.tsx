import { useState } from 'react'
import { toast } from 'sonner'
import { LoaderCircleIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
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
  const login = useUserStore(state => state.login)
  const isLoggingIn = useUserStore(state => state.isLoggingIn)
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen) {
      setUsername('')
      setPassword('')
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
      await login({ username: nextUsername, password: nextPassword })
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
