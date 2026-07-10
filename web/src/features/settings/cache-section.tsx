import type { UseQueryResult } from '@tanstack/react-query'
import { FolderOpenIcon, HardDriveIcon, LoaderCircleIcon, Trash2Icon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import type { ReaderCacheStatsResult } from '@/lib/api/reader'
import { formatBytes } from '@/lib/format'
import { READER_CACHE_LIMITS_MB } from '@/stores/settings-store'
import { SettingRow, SettingsSection } from './shared'

export function CacheSection({
  readerCacheLimitMb,
  stats,
  isOpeningCacheDir,
  isClearingCache,
  onCacheLimitChange,
  onOpenCacheDir,
  onClearCache
}: {
  readerCacheLimitMb: number
  stats: UseQueryResult<ReaderCacheStatsResult, Error>
  isOpeningCacheDir: boolean
  isClearingCache: boolean
  onCacheLimitChange: (limitMb: number) => void
  onOpenCacheDir: () => void
  onClearCache: () => void
}) {
  return (
    <SettingsSection icon={<HardDriveIcon className="size-4" />} title="缓存">
      <SettingRow title="当前缓存大小" description="已解码图片当前占用的磁盘空间">
        <CacheSize stats={stats} />
      </SettingRow>
      <SettingRow title="缓存大小设置" description="超过上限后会自动清理较旧的图片缓存">
        <Select
          value={String(readerCacheLimitMb)}
          onValueChange={value => onCacheLimitChange(Number(value))}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {READER_CACHE_LIMITS_MB.map(limit => (
                <SelectItem key={limit} value={String(limit)}>
                  {formatCacheLimit(limit)}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </SettingRow>
      <SettingRow title="缓存路径" description="缓存在应用目录中的路径">
        <div className="flex items-center gap-2">
          <Input disabled value={cacheDirValue(stats)} title={stats.data?.cacheDir ?? ''} />
          <Button
            type="button"
            variant="outline"
            size="icon"
            disabled={isOpeningCacheDir}
            onClick={onOpenCacheDir}
          >
            {isOpeningCacheDir ? (
              <LoaderCircleIcon className="size-4 animate-spin" />
            ) : (
              <FolderOpenIcon className="size-4" />
            )}
          </Button>
        </div>
      </SettingRow>
      <SettingRow title="清理缓存" description="删除已解码的图片缓存">
        <Button
          type="button"
          variant="destructive"
          size="sm"
          disabled={isClearingCache || stats.data?.totalBytes === 0}
          onClick={onClearCache}
        >
          {isClearingCache ? (
            <LoaderCircleIcon className="size-4 animate-spin" />
          ) : (
            <Trash2Icon className="size-4" />
          )}
          清理缓存
        </Button>
      </SettingRow>
    </SettingsSection>
  )
}

function CacheSize({ stats }: { stats: UseQueryResult<ReaderCacheStatsResult, Error> }) {
  if (stats.isLoading) {
    return <span className="text-sm text-muted-foreground">正在计算</span>
  }

  if (stats.isError) {
    return <span className="text-sm text-destructive">读取失败</span>
  }

  if (!stats.data) {
    return <span className="text-sm text-muted-foreground">0 B</span>
  }

  return (
    <div className="text-right">
      <div className="text-sm font-medium">{formatBytes(stats.data.totalBytes)}</div>
      <div className="mt-1 text-xs text-muted-foreground">{stats.data.fileCount} 个文件</div>
    </div>
  )
}

function cacheDirValue(stats: UseQueryResult<ReaderCacheStatsResult, Error>) {
  if (stats.isLoading) {
    return '正在读取路径'
  }

  if (stats.isError) {
    return '读取失败'
  }

  return stats.data?.cacheDir ?? ''
}

function formatCacheLimit(limitMb: number) {
  return limitMb >= 1024 ? `${limitMb / 1024} GB` : `${limitMb} MB`
}
