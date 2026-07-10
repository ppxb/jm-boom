import { GlobeCheckIcon } from 'lucide-react'

import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import { PROXY_MODES } from '@/stores/settings-store'
import { SettingRow, SettingsSection } from './shared'

export function ProxySection({
  proxyMode,
  proxyHost,
  proxyPort,
  onProxyModeChange,
  onProxyHostChange,
  onProxyPortChange
}: {
  proxyMode: string
  proxyHost: string
  proxyPort: number
  onProxyModeChange: (mode: string) => void
  onProxyHostChange: (host: string) => void
  onProxyPortChange: (port: number) => void
}) {
  return (
    <SettingsSection icon={<GlobeCheckIcon className="size-4" />} title="代理">
      <SettingRow title="本地代理" description="为接口和阅读图片请求启用本机 HTTP 或 SOCKS5 代理">
        <div className="flex items-center gap-2">
          <Select value={proxyMode} onValueChange={onProxyModeChange}>
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {PROXY_MODES.map(mode => (
                  <SelectItem key={mode} value={mode}>
                    {formatProxyMode(mode)}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <Input
            value={proxyHost}
            disabled={proxyMode === 'off'}
            onChange={event => onProxyHostChange(event.target.value)}
            className="w-36"
            placeholder="127.0.0.1"
          />
          <Input
            value={String(proxyPort)}
            disabled={proxyMode === 'off'}
            onChange={event => onProxyPortChange(Number(event.target.value))}
            className="w-24"
            inputMode="numeric"
            min={1}
            max={65535}
            placeholder="7890"
            type="number"
          />
        </div>
      </SettingRow>
    </SettingsSection>
  )
}

function formatProxyMode(mode: string) {
  if (mode === 'http') {
    return 'HTTP'
  }

  if (mode === 'socks5') {
    return 'SOCKS5'
  }

  return '关闭'
}
