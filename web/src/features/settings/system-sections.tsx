import { DatabaseIcon, LoaderCircleIcon, PackageIcon, Trash2Icon } from 'lucide-react'

import { ConfirmDialog } from '@/components/confirm-dialog'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import type { SystemInfo } from '@/lib/api/setting'
import { formatBytes, formatNumber } from '@/lib/format'
import { WEB_VERSION } from '@/lib/version'
import { SettingRow, SettingsSection } from './shared'

export function CacheSection({
  info,
  isLoading,
  isClearing,
  onClear
}: {
  info: SystemInfo | undefined
  isLoading: boolean
  isClearing: boolean
  onClear: () => void
}) {
  const cache = info?.cache
  const sizeLabel = isLoading
    ? '正在读取'
    : cache
      ? `${formatBytes(cache.sizeBytes)} / ${formatBytes(cache.maxSizeBytes)}`
      : '读取失败'
  const description = cache
    ? `Server 已缓存 ${formatNumber(cache.entryCount)} 个文件`
    : '统计 Server 保存的封面及阅读图片缓存'

  return (
    <SettingsSection icon={<DatabaseIcon className="size-4" />} title="缓存">
      <SettingRow title="服务端缓存" description={description}>
        <div className="flex items-center justify-between gap-3 sm:justify-end">
          <span className="text-sm font-medium tabular-nums">{sizeLabel}</span>
          <ConfirmDialog
            title="清除服务端缓存？"
            description="将删除 Server 已保存的封面及阅读图片，后续访问时会重新下载，不会删除下载任务记录。"
            confirmText="确认清除"
            variant="destructive"
            loading={isClearing}
            onConfirm={onClear}
            icon={<Trash2Icon className="size-5 text-destructive" />}
            trigger={
              <Button
                type="button"
                variant="destructive"
                size="sm"
                disabled={isLoading || isClearing}
              >
                {isClearing ? (
                  <LoaderCircleIcon className="size-4 animate-spin" />
                ) : (
                  <Trash2Icon className="size-4" />
                )}
                清除缓存
              </Button>
            }
          />
        </div>
      </SettingRow>
    </SettingsSection>
  )
}

export function VersionSection({
  info,
  isLoading
}: {
  info: SystemInfo | undefined
  isLoading: boolean
}) {
  const serverVersion = isLoading
    ? '读取中'
    : info?.serverVersion
      ? `v${info.serverVersion}`
      : '不可用'

  return (
    <SettingsSection icon={<PackageIcon className="size-4" />} title="版本">
      <div className="space-y-5">
        <SettingRow title="Web" description="当前浏览器端界面版本" inline>
          <Badge variant="outline">v{WEB_VERSION}</Badge>
        </SettingRow>
        <SettingRow title="Server" description="当前服务端程序版本" inline>
          <Badge variant="outline">{serverVersion}</Badge>
        </SettingRow>
      </div>
    </SettingsSection>
  )
}
