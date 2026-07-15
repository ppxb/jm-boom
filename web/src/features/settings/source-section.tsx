import { DatabaseIcon } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import type { InstalledSource } from '@/lib/api/source'
import { SettingRow, SettingsSection } from './shared'

export function SourceSection({
  sources,
  selectedSourceId,
  isLoading,
  onSourceChange
}: {
  sources: InstalledSource[]
  selectedSourceId: string | null
  isLoading: boolean
  onSourceChange: (sourceId: string) => void
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
