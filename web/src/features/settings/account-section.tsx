import {
  BadgeAlert,
  BadgeCheck,
  KeyRoundIcon,
  LoaderCircleIcon,
  SaveIcon,
  Trash2Icon
} from 'lucide-react'
import { useEffect, useState } from 'react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import type { AccountState } from '@/lib/api/setting'
import { SettingRow, SettingsSection } from './shared'

export function AccountSection({
  state,
  isLoading,
  isSaving,
  isClearing,
  onSave,
  onClear
}: {
  state: AccountState | undefined
  isLoading: boolean
  isSaving: boolean
  isClearing: boolean
  onSave: (input: {
    username: string
    password?: string
    autoLogin: boolean
    autoSignIn: boolean
  }) => void
  onClear: () => void
}) {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [autoLogin, setAutoLogin] = useState(true)
  const [autoSignIn, setAutoSignIn] = useState(true)

  useEffect(() => {
    setUsername(state?.username ?? '')
    setPassword('')
    setAutoLogin(state?.autoLogin ?? true)
    setAutoSignIn(state?.autoSignIn ?? true)
  }, [state])

  const isBusy = isLoading || isSaving || isClearing
  const canSave = username.trim().length > 0 && (password.trim().length > 0 || state?.username)

  return (
    <SettingsSection icon={<KeyRoundIcon className="size-4" />} title="账号">
      <div className="space-y-5">
        <SettingRow title="账号密码" description="保存后用于自动登录禁漫天堂">
          <div className="grid gap-2 sm:min-w-96 sm:grid-cols-2">
            <Input
              value={username}
              disabled={isBusy}
              onChange={event => setUsername(event.target.value)}
              autoComplete="username"
              className="placeholder:text-sm"
              placeholder="用户名或邮箱"
            />
            <Input
              value={password}
              disabled={isBusy}
              onChange={event => setPassword(event.target.value)}
              autoComplete="current-password"
              className="placeholder:text-sm"
              placeholder={state?.username ? '留空保持原密码' : '密码'}
              type="password"
            />
          </div>
        </SettingRow>

        <SettingRow title="登录状态" description="当前账号登录状态" inline>
          <Badge variant="outline" className="h-7 min-w-20 px-3 text-sm [&>svg]:size-4!">
            {state?.loginStatus === 'loggedIn' ? (
              <BadgeCheck data-icon="inline-start" />
            ) : (
              <BadgeAlert data-icon="inline-start" />
            )}
            {loginStatusLabel(state?.loginStatus)}
          </Badge>
        </SettingRow>

        <SettingRow title="签到状态" description="自动签到只在登录成功后执行" inline>
          <Badge variant="outline" className="h-7 min-w-20 px-3 text-sm [&>svg]:size-4!">
            {state?.signInStatus === 'signedIn' ? (
              <BadgeCheck data-icon="inline-start" />
            ) : (
              <BadgeAlert data-icon="inline-start" />
            )}
            {signInStatusLabel(state?.signInStatus)}
          </Badge>
        </SettingRow>

        <SettingRow title="自动登录" description="自动登录已保存的账号" inline>
          <Switch checked={autoLogin} disabled={isBusy} onCheckedChange={setAutoLogin} />
        </SettingRow>

        <SettingRow title="自动签到" description="登录成功后自动完成当天签到" inline>
          <Switch checked={autoSignIn} disabled={isBusy} onCheckedChange={setAutoSignIn} />
        </SettingRow>

        <div className="flex items-center justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={isBusy || !canSave}
            onClick={() =>
              onSave({ username, password: password || undefined, autoLogin, autoSignIn })
            }
          >
            {isSaving ? (
              <LoaderCircleIcon className="size-4 animate-spin" />
            ) : (
              <SaveIcon className="size-4" />
            )}
            保存并登录
          </Button>
          <Button
            type="button"
            variant="destructive"
            size="sm"
            disabled={isBusy || !state?.username}
            onClick={onClear}
          >
            {isClearing ? (
              <LoaderCircleIcon className="size-4 animate-spin" />
            ) : (
              <Trash2Icon className="size-4" />
            )}
            退出登录
          </Button>
        </div>
      </div>
    </SettingsSection>
  )
}

function loginStatusLabel(status: AccountState['loginStatus'] | undefined) {
  if (status === 'loggedIn') return '已登录'
  if (status === 'loggingIn') return '正在登录'
  return '未登录'
}

function signInStatusLabel(status: AccountState['signInStatus'] | undefined) {
  if (status === 'signedIn') return '已签到'
  if (status === 'signingIn') return '正在签到'
  return '待签到'
}
