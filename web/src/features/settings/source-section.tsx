import { useEffect, useMemo, useState } from 'react'
import {
  CheckIcon,
  DatabaseIcon,
  DownloadIcon,
  ImageIcon,
  LoaderCircleIcon,
  SearchIcon
} from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger
} from '@/components/ui/dialog'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import type { AvailableSource, InstalledSource } from '@/lib/api/source'
import { SettingRow, SettingsSection } from './shared'

type ManagedSource = AvailableSource & {
  installedVersion: number | null
}

type SourceGroup = {
  language: string
  sources: ManagedSource[]
}

const ALL_FILTER = 'all'

export function SourceSection({
  sources,
  isLoading,
  catalog,
  isCatalogLoading,
  catalogError,
  installingSourceId,
  onInstall
}: {
  sources: InstalledSource[]
  isLoading: boolean
  catalog: AvailableSource[]
  isCatalogLoading: boolean
  catalogError: unknown
  installingSourceId: string | null
  onInstall: (sourceId: string) => void
}) {
  return (
    <SettingsSection icon={<DatabaseIcon className="size-4" />} title="漫画源">
      <SettingRow title="源管理" description="管理聚合搜索、详情和阅读使用的漫画源">
        <SourceManagerDialog
          sources={sources}
          isLoading={isLoading}
          catalog={catalog}
          isCatalogLoading={isCatalogLoading}
          catalogError={catalogError}
          installingSourceId={installingSourceId}
          onInstall={onInstall}
        />
      </SettingRow>
    </SettingsSection>
  )
}

function SourceManagerDialog({
  sources,
  isLoading,
  catalog,
  isCatalogLoading,
  catalogError,
  installingSourceId,
  onInstall
}: {
  sources: InstalledSource[]
  isLoading: boolean
  catalog: AvailableSource[]
  isCatalogLoading: boolean
  catalogError: unknown
  installingSourceId: string | null
  onInstall: (sourceId: string) => void
}) {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const [language, setLanguage] = useState(ALL_FILTER)
  const [contentRating, setContentRating] = useState(ALL_FILTER)
  const managedSources = useMemo(() => mergeSources(catalog, sources), [catalog, sources])
  const languages = useMemo(
    () =>
      [
        ...new Set(
          managedSources
            .map(sourceLanguage)
            .filter(language => language !== 'multi' && language !== 'unknown')
        )
      ].sort((left, right) =>
        formatLanguage(left).localeCompare(formatLanguage(right), 'zh-CN')
      ),
    [managedSources]
  )
  const groups = useMemo(
    () => groupSources(filterSources(managedSources, query, language, contentRating)),
    [contentRating, language, managedSources, query]
  )
  const visibleCount = groups.reduce((count, group) => count + group.sources.length, 0)

  useEffect(() => {
    if (open) return
    setQuery('')
    setLanguage(ALL_FILTER)
    setContentRating(ALL_FILTER)
  }, [open])

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button type="button" variant="outline" disabled={isLoading}>
          {isLoading ? <LoaderCircleIcon className="size-4 animate-spin" /> : <DatabaseIcon />}
          已安装 {sources.length}
        </Button>
      </DialogTrigger>

      <DialogContent className="h-auto max-h-none gap-0 overflow-hidden p-0 sm:max-w-2xl">
        <DialogHeader className="px-5 pt-5 pr-12 pb-4 sm:px-6 sm:pt-6 sm:pr-14">
          <DialogTitle>源管理</DialogTitle>
          <DialogDescription>
            从可信目录安装漫画源；所有已安装源都会参与聚合搜索。
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-2 border-y px-5 py-4 sm:grid-cols-[minmax(0,1fr)_140px_160px] sm:px-6">
          <InputGroup>
            <InputGroupAddon>
              <SearchIcon className="size-4" />
            </InputGroupAddon>
            <InputGroupInput
              value={query}
              onChange={event => setQuery(event.target.value)}
              placeholder="搜索源名称"
              aria-label="搜索源名称"
            />
          </InputGroup>

          <Select value={language} onValueChange={setLanguage}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="全部语言" />
            </SelectTrigger>
            <SelectContent position="popper" align="end">
              <SelectItem value={ALL_FILTER}>全部语言</SelectItem>
              {languages.map(item => (
                <SelectItem key={item} value={item}>
                  {formatLanguage(item)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>

          <Select value={contentRating} onValueChange={setContentRating}>
            <SelectTrigger className="w-full">
              <SelectValue placeholder="全部分级" />
            </SelectTrigger>
            <SelectContent position="popper" align="end">
              <SelectItem value={ALL_FILTER}>全部分级</SelectItem>
              <SelectItem value="1">安全</SelectItem>
              <SelectItem value="2">包含 NSFW</SelectItem>
              <SelectItem value="3">NSFW</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex items-center justify-between gap-3 px-5 py-3 text-xs text-muted-foreground sm:px-6">
          <span>显示 {visibleCount} 个源</span>
          {isCatalogLoading ? (
            <span className="inline-flex items-center gap-1.5">
              <LoaderCircleIcon className="size-3.5 animate-spin" />
              正在刷新目录
            </span>
          ) : null}
        </div>

        <div className="h-[38vh] min-h-52 max-h-80 touch-pan-y overflow-y-scroll overscroll-contain sm:h-[46vh] sm:max-h-96">
          <div className="px-5 pb-5 sm:px-6 sm:pb-6">
            {catalogError instanceof Error ? (
              <div className="mb-4 rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-3 text-sm text-destructive">
                源目录暂时不可用：{catalogError.message}
              </div>
            ) : null}

            {groups.length > 0 ? (
              <div className="space-y-6">
                {groups.map(group => (
                  <SourceLanguageGroup
                    key={group.language}
                    group={group}
                    installingSourceId={installingSourceId}
                    onInstall={onInstall}
                  />
                ))}
              </div>
            ) : isCatalogLoading ? (
              <SourceListSkeleton />
            ) : (
              <div className="rounded-xl border border-dashed px-4 py-10 text-center text-sm text-muted-foreground">
                没有符合条件的漫画源
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}

function SourceLanguageGroup({
  group,
  installingSourceId,
  onInstall
}: {
  group: SourceGroup
  installingSourceId: string | null
  onInstall: (sourceId: string) => void
}) {
  return (
    <section className="space-y-2.5">
      <div className="flex items-center gap-2">
        <h3 className="text-sm font-semibold">{formatLanguage(group.language)}</h3>
        <Badge variant="outline">{group.sources.length}</Badge>
      </div>

      <div className="space-y-2">
        {group.sources.map(source => (
          <SourceManagerItem
            key={source.id}
            source={source}
            isInstalling={installingSourceId === source.id}
            onInstall={onInstall}
          />
        ))}
      </div>
    </section>
  )
}

function SourceManagerItem({
  source,
  isInstalling,
  onInstall
}: {
  source: ManagedSource
  isInstalling: boolean
  onInstall: (sourceId: string) => void
}) {
  const isInstalled = source.installedVersion != null
  const isCurrent = isInstalled && source.installedVersion! >= source.version
  const canInstall = source.downloadUrl != null && !isCurrent

  return (
    <div className="flex items-center gap-3.5 rounded-xl border bg-card/50 px-3.5 py-3">
      <SourceIcon source={source} />

      <div className="min-w-0 flex-1 space-y-1.5">
        <div className="truncate text-sm font-medium">{source.name}</div>
        <div className="flex flex-wrap items-center gap-1.5">
          <Badge variant="outline">v{source.version}</Badge>
          <ContentRatingBadge rating={source.contentRating} />
        </div>
      </div>

      <Button
        type="button"
        size="sm"
        variant="outline"
        className="shrink-0"
        disabled={!canInstall || isInstalling}
        onClick={() => onInstall(source.id)}
      >
        {isInstalling ? (
          <LoaderCircleIcon className="size-4 animate-spin" />
        ) : isCurrent ? (
          <CheckIcon className="size-4" />
        ) : (
          <DownloadIcon className="size-4" />
        )}
        {source.downloadUrl == null
          ? isCurrent
            ? '已安装'
            : '不可用'
          : !isInstalled
            ? '安装'
            : isCurrent
              ? '已安装'
              : '升级'}
      </Button>
    </div>
  )
}

function SourceIcon({ source }: { source: ManagedSource }) {
  const [failed, setFailed] = useState(false)

  useEffect(() => setFailed(false), [source.iconUrl])

  return (
    <div className="relative flex size-14 shrink-0 items-center justify-center overflow-hidden rounded-xl border bg-muted text-muted-foreground sm:size-16">
      <ImageIcon className="size-6" />
      {source.iconUrl && !failed ? (
        <img
          src={source.iconUrl}
          alt=""
          loading="lazy"
          decoding="async"
          referrerPolicy="no-referrer"
          className="absolute inset-0 size-full object-cover"
          onError={() => setFailed(true)}
        />
      ) : null}
    </div>
  )
}

function ContentRatingBadge({ rating }: { rating: number }) {
  const label = formatContentRating(rating)
  const className =
    rating === 1
      ? 'border-emerald-600/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300'
      : rating === 2
        ? 'border-amber-600/30 bg-amber-500/10 text-amber-700 dark:text-amber-300'
        : rating === 3
          ? 'border-destructive/30 bg-destructive/10 text-destructive'
          : undefined

  return (
    <Badge variant="outline" className={className}>
      {label}
    </Badge>
  )
}

function SourceListSkeleton() {
  return (
    <div className="space-y-2">
      {Array.from({ length: 4 }).map((_, index) => (
        <div key={index} className="h-20 animate-pulse rounded-xl bg-muted" />
      ))}
    </div>
  )
}

function mergeSources(catalog: AvailableSource[], installed: InstalledSource[]) {
  const installedById = new Map(installed.map(source => [source.info.id, source]))
  const merged = catalog.map(source => ({
    ...source,
    installedVersion: installedById.get(source.id)?.info.version ?? source.installedVersion
  }))
  const catalogIds = new Set(catalog.map(source => source.id))

  for (const source of installed) {
    if (catalogIds.has(source.info.id)) continue
    merged.push({
      id: source.info.id,
      name: source.info.name,
      version: source.info.version,
      iconUrl: null,
      downloadUrl: null,
      languages: source.info.languages,
      contentRating: source.info.contentRating ?? 0,
      installedVersion: source.info.version
    })
  }

  return merged
}

function filterSources(
  sources: ManagedSource[],
  query: string,
  language: string,
  contentRating: string
) {
  const keyword = query.trim().toLocaleLowerCase('zh-CN')
  return sources.filter(source => {
    if (keyword && !source.name.toLocaleLowerCase('zh-CN').includes(keyword)) return false
    const sourceLanguageCode = sourceLanguage(source)
    if (
      language !== ALL_FILTER &&
      sourceLanguageCode !== language &&
      sourceLanguageCode !== 'multi'
    ) {
      return false
    }
    if (contentRating !== ALL_FILTER && source.contentRating !== Number(contentRating)) return false
    return true
  })
}

function groupSources(sources: ManagedSource[]): SourceGroup[] {
  const groups = new Map<string, ManagedSource[]>()
  for (const source of sources) {
    const language = sourceLanguage(source)
    const items = groups.get(language) ?? []
    items.push(source)
    groups.set(language, items)
  }

  return [...groups.entries()]
    .map(([language, items]) => ({
      language,
      sources: items.sort((left, right) => left.name.localeCompare(right.name, 'zh-CN'))
    }))
    .sort((left, right) =>
      formatLanguage(left.language).localeCompare(formatLanguage(right.language), 'zh-CN')
    )
}

function sourceLanguage(source: ManagedSource) {
  const prefix = source.id.split('.', 1)[0]?.trim().toLowerCase()
  if (!prefix) return 'unknown'
  if (prefix === 'multi') return 'multi'
  if (prefix.startsWith('zh-') || prefix === 'cn') return 'zh'
  return prefix
}

function formatLanguage(language: string) {
  const normalized = language.toLowerCase()
  const labels: Record<string, string> = {
    zh: '中文',
    'zh-hans': '简体中文',
    'zh-hant': '繁体中文',
    en: '英语',
    ja: '日语',
    ko: '韩语',
    es: '西班牙语',
    fr: '法语',
    de: '德语',
    ru: '俄语',
    multi: '多语言',
    unknown: '未声明语言'
  }
  return labels[normalized] ?? language.toUpperCase()
}

function formatContentRating(rating: number) {
  switch (rating) {
    case 1:
      return '安全'
    case 2:
      return '包含 NSFW'
    case 3:
      return 'NSFW'
    default:
      return '未分级'
  }
}
