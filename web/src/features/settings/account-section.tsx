import { KeyRoundIcon } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'

import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import type { SavedLoginConfig } from '@/lib/api/user'
import { SettingRow, SettingsSection } from './shared'

export function AccountSection({
  savedLoginConfig,
  isLoading,
  isSaving,
  isSettingAutoLogin,
  onAutoLoginChange,
  onCredentialsChange
}: {
  savedLoginConfig: SavedLoginConfig | null | undefined
  isLoading: boolean
  isSaving: boolean
  isSettingAutoLogin: boolean
  onAutoLoginChange: (autoLogin: boolean) => void
  onCredentialsChange: (input: { username: string; password: string; autoLogin: boolean }) => void
}) {
  const [autoLogin, setAutoLogin] = useState(false)
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const hasHydratedRef = useRef(false)
  const onCredentialsChangeRef = useRef(onCredentialsChange)

  useEffect(() => {
    onCredentialsChangeRef.current = onCredentialsChange
  }, [onCredentialsChange])

  useEffect(() => {
    setAutoLogin(savedLoginConfig?.autoLogin ?? false)
    setUsername(savedLoginConfig?.username ?? '')
    setPassword('')
    hasHydratedRef.current = true
  }, [savedLoginConfig])

  useEffect(() => {
    if (!hasHydratedRef.current || !autoLogin) {
      return
    }

    const nextUsername = username.trim()
    const nextPassword = password.trim()

    if (!nextUsername || !nextPassword) {
      return
    }

    const timeoutId = window.setTimeout(() => {
      onCredentialsChangeRef.current({
        username: nextUsername,
        password: nextPassword,
        autoLogin
      })
    }, 700)

    return () => window.clearTimeout(timeoutId)
  }, [autoLogin, password, username])

  const isBusy = isLoading || isSaving || isSettingAutoLogin

  function handleAutoLoginChange(nextAutoLogin: boolean) {
    setAutoLogin(nextAutoLogin)

    if (!nextAutoLogin || savedLoginConfig?.hasPassword) {
      onAutoLoginChange(nextAutoLogin)
    }
  }

  return (
    <SettingsSection icon={<KeyRoundIcon className="size-4" />} title="账号">
      <SettingRow
        title="启用自动登录"
        description="启动 APP 时使用保存的账号密码重新登录，并刷新用户资料和会话"
      >
        <Switch checked={autoLogin} disabled={isBusy} onCheckedChange={handleAutoLoginChange} />
      </SettingRow>

      {autoLogin ? (
        <SettingRow title="账号密码" description="账号密码会加密保存到 SQLite，用于自动登录">
          <div className="grid w-[360px] grid-cols-2 items-center gap-2">
            <Input
              value={username}
              disabled={isBusy}
              onChange={event => setUsername(event.target.value)}
              autoComplete="username"
              placeholder="用户名或邮箱"
            />
            <Input
              value={password}
              disabled={isBusy}
              onChange={event => setPassword(event.target.value)}
              autoComplete="current-password"
              placeholder={savedLoginConfig?.hasPassword ? '输入新密码' : '密码'}
              type="password"
            />
          </div>
        </SettingRow>
      ) : null}
    </SettingsSection>
  )
}
