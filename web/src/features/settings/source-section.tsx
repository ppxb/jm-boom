import { DatabaseIcon, DownloadIcon, LoaderCircleIcon } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import type { AvailableSource, InstalledSource } from '@/lib/api/source'
import { SettingRow, SettingsSection } from './shared'

export function SourceSection({
  sources,
  selectedSourceId,
  isLoading,
  onSourceChange,
  catalog,
  isCatalogLoading,
  catalogError,
  installingSourceId,
  onInstall
}: {
  sources: InstalledSource[]
  selectedSourceId: string | null
  isLoading: boolean
  onSourceChange: (sourceId: string) => void
  catalog: AvailableSource[]
  isCatalogLoading: boolean
  catalogError: unknown
  installingSourceId: string | null
  onInstall: (sourceId: string) => void
}) {
  const selected = sources.find(source => source.info.id === selectedSourceId)

  return (
    <SettingsSection icon={<DatabaseIcon className="size-4" />} title="漫画源">
      <SettingRow
        title="当前源"
        description="通用源迁移入口；完成迁移的页面将通过所选源加载"
      >
        <Select
          value={selectedSourceId ?? undefined}
          disabled={isLoading || sources.length === 0}
          onValueChange={onSourceChange}
        >
          <SelectTrigger className="w-full sm:w-72">
            <SelectValue placeholder={isLoading ? '正在加载漫画源' : '未安装漫画源'} />
          </SelectTrigger>
          <SelectContent position="popper" align="end">
            <SelectGroup>
              {sources.map(source => (
                <SelectItem key={source.info.id} value={source.info.id}>
                  <span className="flex items-center gap-2">
                    <span>{source.info.name}</span>
                    <span className="text-xs text-muted-foreground">v{source.info.version}</span>
                  </span>
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </SettingRow>

      {selected ? (
        <SettingRow
          title="源能力"
          description={`${selected.info.id} · ${formatLanguages(selected.info.languages)}`}
        >
          <div className="flex max-w-sm flex-wrap justify-end gap-1.5">
            {capabilityLabels(selected).map(label => (
              <Badge key={label} variant="outline">
                {label}
              </Badge>
            ))}
          </div>
        </SettingRow>
      ) : null}

      <SettingRow
        title="可安装源"
        description={
          catalogError instanceof Error
            ? `目录暂时不可用：${catalogError.message}`
            : '从服务端配置的可信目录安装并校验 .aix 源包'
        }
      >
        {isCatalogLoading ? (
          <LoaderCircleIcon className="size-4 animate-spin text-muted-foreground" />
        ) : catalog.length > 0 ? (
          <div className="max-h-72 w-full space-y-2 overflow-y-auto sm:w-96">
            {catalog.map(source => (
              <div
                key={source.id}
                className="flex items-center justify-between gap-3 rounded-lg border px-3 py-2"
              >
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">{source.name}</div>
                  <div className="truncate text-xs text-muted-foreground">
                    {source.id} · v{source.version}
                    {source.installedVersion != null ? ` · 已安装 v${source.installedVersion}` : ''}
                  </div>
                </div>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={
                    source.downloadUrl == null ||
                    (source.installedVersion != null &&
                      source.installedVersion >= source.version) ||
                    installingSourceId === source.id
                  }
                  onClick={() => onInstall(source.id)}
                >
                  {installingSourceId === source.id ? (
                    <LoaderCircleIcon className="size-4 animate-spin" />
                  ) : (
                    <DownloadIcon className="size-4" />
                  )}
                  {source.downloadUrl == null
                    ? '不可用'
                    : source.installedVersion == null
                    ? '安装'
                    : source.installedVersion < source.version
                      ? '升级'
                      : '已安装'}
                </Button>
              </div>
            ))}
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">暂无可安装源</span>
        )}
      </SettingRow>
    </SettingsSection>
  )
}

function capabilityLabels(source: InstalledSource) {
  const labels: string[] = []
  if (source.capabilities.providesHome) labels.push('首页')
  if (source.capabilities.providesListings) labels.push('列表')
  if (source.filterCount > 0 || source.capabilities.dynamicFilters) labels.push('筛选')
  if (source.capabilities.providesImageRequests) labels.push('图片请求')
  if (source.capabilities.processesPages) labels.push('页面处理')
  if (source.capabilities.usesCanvas) labels.push('画布')
  return labels.length > 0 ? labels : ['基础搜索与阅读']
}

function formatLanguages(languages: string[]) {
  return languages.length > 0 ? languages.join(', ') : '未声明语言'
}
