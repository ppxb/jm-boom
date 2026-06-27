import { useMutation, useQuery, useQueryClient, type UseQueryResult } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import {
  CheckCircle2Icon,
  Trash2Icon,
  FolderOpenIcon,
  HardDriveIcon,
  LoaderCircleIcon,
  MonitorCogIcon,
  MoonIcon,
  NetworkIcon,
  RotateCcwIcon,
  RefreshCwIcon,
  SunIcon,
  XCircleIcon,
  ShieldIcon,
  GlobeCheckIcon,
  TvMinimalPlayIcon
} from 'lucide-react'
import { useTheme } from 'next-themes'
import { useEffect, useMemo, useRef, type ReactNode } from 'react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { discoverApiEndpoints, type ApiEndpointProbe } from '@/lib/api/setting'
import {
  clearReaderCache,
  getReaderCacheStats,
  openReaderCacheDir,
  type ReaderCacheStatsResult
} from '@/lib/api/reader'
import { cn } from '@/lib/utils'
import {
  FALLBACK_API_ENDPOINTS,
  IMAGE_SHUNTS,
  PREFETCH_COUNTS,
  PROXY_MODES,
  READER_CACHE_LIMITS_MB,
  useSettingsStore
} from '@/stores/settings-store'

export const Route = createFileRoute('/_app/settings')({
  component: SettingsPage
})

const THEME_OPTIONS = [
  { value: 'system', label: '跟随系统', icon: MonitorCogIcon },
  { value: 'light', label: '日间模式', icon: SunIcon },
  { value: 'dark', label: '夜间模式', icon: MoonIcon }
]

function SettingsPage() {
  const queryClient = useQueryClient()
  const { theme = 'system', setTheme } = useTheme()
  const api = useSettingsStore(state => state.api)
  const shunt = useSettingsStore(state => state.shunt)
  const prefetchCount = useSettingsStore(state => state.prefetchCount)
  const readerCacheLimitMb = useSettingsStore(state => state.readerCacheLimitMb)
  const cacheLimitBytes = readerCacheLimitMb * 1024 * 1024
  const proxyMode = useSettingsStore(state => state.proxyMode)
  const proxyHost = useSettingsStore(state => state.proxyHost)
  const proxyPort = useSettingsStore(state => state.proxyPort)
  const hideCovers = useSettingsStore(state => state.hideCovers)
  const setApi = useSettingsStore(state => state.setApi)
  const setShunt = useSettingsStore(state => state.setShunt)
  const setPrefetchCount = useSettingsStore(state => state.setPrefetchCount)
  const setReaderCacheLimitMb = useSettingsStore(state => state.setReaderCacheLimitMb)
  const setProxyMode = useSettingsStore(state => state.setProxyMode)
  const setProxyHost = useSettingsStore(state => state.setProxyHost)
  const setProxyPort = useSettingsStore(state => state.setProxyPort)
  const setHideCovers = useSettingsStore(state => state.setHideCovers)
  const reset = useSettingsStore(state => state.reset)
  const endpointDiscovery = useQuery({
    queryKey: ['jm-api-endpoint-discovery'],
    queryFn: discoverApiEndpoints,
    staleTime: 60 * 1000,
    gcTime: 5 * 60 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })
  const endpointOptions = useEndpointOptions(api, endpointDiscovery.data)
  const apiRef = useRef(api)
  const lastPreferredDiscoveryAtRef = useRef(0)
  const readerCacheStats = useQuery({
    queryKey: ['reader-cache-stats', cacheLimitBytes],
    queryFn: () => getReaderCacheStats(cacheLimitBytes),
    staleTime: 0,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false
  })
  const clearCache = useMutation({
    mutationFn: () => clearReaderCache(cacheLimitBytes),
    onSuccess: data => {
      toast.success('阅读缓存已清理')
      queryClient.setQueryData(['reader-cache-stats', cacheLimitBytes], data)
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })
  const openCacheDir = useMutation({
    mutationFn: openReaderCacheDir,
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  useEffect(() => {
    apiRef.current = api
  }, [api])

  useEffect(() => {
    if (
      !endpointDiscovery.data ||
      endpointDiscovery.dataUpdatedAt === 0 ||
      lastPreferredDiscoveryAtRef.current === endpointDiscovery.dataUpdatedAt
    ) {
      return
    }

    lastPreferredDiscoveryAtRef.current = endpointDiscovery.dataUpdatedAt

    const preferredEndpoint = findPreferredEndpoint(endpointDiscovery.data)

    if (preferredEndpoint && apiRef.current !== preferredEndpoint.endpoint) {
      setApi(preferredEndpoint.endpoint)
    }
  }, [endpointDiscovery.data, endpointDiscovery.dataUpdatedAt, setApi])

  function resetSettings() {
    reset()
    setTheme('system')
    toast.success('设置已恢复默认')
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-5xl space-y-8 p-[96px_32px_32px_96px]">
        <header className="flex items-end justify-between gap-4">
          <div>
            <h1 className="text-3xl font-semibold tracking-normal">设置</h1>
            <p className="mt-2 text-sm text-muted-foreground">调整 APP 配置和内容显示偏好</p>
          </div>
          <Button variant="outline" size="sm" onClick={resetSettings} className="text-xs">
            <RotateCcwIcon className="size-4" />
            恢复默认
          </Button>
        </header>

        <Card>
          <CardHeader>
            <CardTitle className="text-base">偏好设置</CardTitle>
          </CardHeader>
          <CardContent className="space-y-8">
            <section className="space-y-5">
              <SectionTitle icon={<MonitorCogIcon className="size-4" />} title="外观" />
              <SettingRow title="主题" description="控制应用的明暗色主题">
                <Tabs value={theme} onValueChange={setTheme}>
                  <TabsList>
                    {THEME_OPTIONS.map(option => (
                      <TabsTrigger key={option.value} value={option.value}>
                        <option.icon className="size-4" />
                      </TabsTrigger>
                    ))}
                  </TabsList>
                </Tabs>
              </SettingRow>
            </section>

            <Separator />

            <section className="space-y-5">
              <SectionTitle icon={<NetworkIcon className="size-4" />} title="网络" />
              <SettingRow title="API 接口" description="测速后自动优选延迟最低的可用接口">
                <div className="flex items-center gap-2">
                  <Select value={api} onValueChange={setApi}>
                    <SelectTrigger>
                      <SelectValue>
                        <EndpointDisplay
                          endpoint={api}
                          probe={endpointOptions.find(option => option.endpoint === api)}
                          isDiscovering={endpointDiscovery.isFetching}
                          compact
                        />
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      <SelectGroup>
                        {endpointOptions.map(option => (
                          <SelectItem
                            key={option.endpoint}
                            value={option.endpoint}
                            textValue={formatEndpoint(option.endpoint)}
                            className="py-2.5"
                          >
                            <EndpointDisplay
                              endpoint={option.endpoint}
                              probe={option}
                              isDiscovering={endpointDiscovery.isFetching}
                            />
                          </SelectItem>
                        ))}
                      </SelectGroup>
                    </SelectContent>
                  </Select>
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    disabled={endpointDiscovery.isFetching}
                    onClick={() => void endpointDiscovery.refetch()}
                  >
                    <RefreshCwIcon
                      className={cn('size-4', endpointDiscovery.isFetching && 'animate-spin')}
                    />
                  </Button>
                </div>
              </SettingRow>

              <SettingRow title="图片线路" description="切换图片加载使用的分流线路">
                <Select value={shunt} onValueChange={setShunt}>
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {IMAGE_SHUNTS.map(option => (
                        <SelectItem key={option} value={option}>
                          线路 {option}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </SettingRow>
            </section>

            <Separator />

            <section className="space-y-5">
              <SectionTitle icon={<GlobeCheckIcon className="size-4" />} title="代理" />
              <SettingRow
                title="本地代理"
                description="为接口和阅读图片请求启用本机 HTTP 或 SOCKS5 代理"
              >
                <div className="flex items-center gap-2">
                  <Select value={proxyMode} onValueChange={setProxyMode}>
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
                    onChange={event => setProxyHost(event.target.value)}
                    className="w-36"
                    placeholder="127.0.0.1"
                  />
                  <Input
                    value={String(proxyPort)}
                    disabled={proxyMode === 'off'}
                    onChange={event => setProxyPort(Number(event.target.value))}
                    className="w-24"
                    inputMode="numeric"
                    min={1}
                    max={65535}
                    placeholder="7890"
                    type="number"
                  />
                </div>
              </SettingRow>
            </section>

            <Separator />

            <section className="space-y-5">
              <SectionTitle icon={<TvMinimalPlayIcon className="size-4" />} title="阅读" />
              <SettingRow title="图片预载数量" description="当前页前后预载的窗口半径">
                <Select
                  value={String(prefetchCount)}
                  onValueChange={value => setPrefetchCount(Number(value))}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {PREFETCH_COUNTS.map(option => (
                        <SelectItem key={option} value={String(option)}>
                          前后各 {option} 张
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </SettingRow>
            </section>

            <Separator />

            <section className="space-y-5">
              <SectionTitle icon={<HardDriveIcon className="size-4" />} title="缓存" />
              <SettingRow title="当前缓存大小" description="已解码图片当前占用的磁盘空间">
                <CacheSize stats={readerCacheStats} />
              </SettingRow>
              <SettingRow title="缓存大小设置" description="超过上限后会自动清理较旧的图片缓存">
                <Select
                  value={String(readerCacheLimitMb)}
                  onValueChange={value => setReaderCacheLimitMb(Number(value))}
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
                  <Input
                    disabled
                    value={
                      readerCacheStats.isLoading
                        ? '正在读取路径'
                        : readerCacheStats.isError
                          ? '读取失败'
                          : (readerCacheStats.data?.cacheDir ?? '')
                    }
                    title={readerCacheStats.data?.cacheDir ?? ''}
                  />
                  <Button
                    type="button"
                    variant="outline"
                    size="icon"
                    disabled={openCacheDir.isPending}
                    onClick={() => openCacheDir.mutate()}
                  >
                    {openCacheDir.isPending ? (
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
                  disabled={clearCache.isPending || readerCacheStats.data?.totalBytes === 0}
                  onClick={() => clearCache.mutate()}
                >
                  {clearCache.isPending ? (
                    <LoaderCircleIcon className="size-4 animate-spin" />
                  ) : (
                    <Trash2Icon className="size-4" />
                  )}
                  清理缓存
                </Button>
              </SettingRow>
            </section>

            <Separator />

            <section className="space-y-5">
              <SectionTitle icon={<ShieldIcon className="size-4" />} title="NSFW 保护" />
              <SettingRow title="封面隐私模式" description="控制列表项是否遮挡封面">
                <Switch checked={hideCovers} onCheckedChange={setHideCovers} />
              </SettingRow>
            </section>
          </CardContent>
        </Card>
      </div>
    </main>
  )
}

function useEndpointOptions(currentEndpoint: string, probes: ApiEndpointProbe[] | undefined) {
  return useMemo(() => {
    const options = new Map<string, ApiEndpointProbe>()

    for (const endpoint of FALLBACK_API_ENDPOINTS) {
      options.set(endpoint, {
        endpoint,
        available: false,
        latencyMs: null,
        imgHost: null,
        error: null
      })
    }

    for (const probe of probes ?? []) {
      options.set(probe.endpoint, probe)
    }

    if (currentEndpoint && !options.has(currentEndpoint)) {
      options.set(currentEndpoint, {
        endpoint: currentEndpoint,
        available: false,
        latencyMs: null,
        imgHost: null,
        error: null
      })
    }

    return [...options.values()].sort((left, right) => {
      if (left.available !== right.available) {
        return left.available ? -1 : 1
      }

      return (
        (left.latencyMs ?? Number.MAX_SAFE_INTEGER) - (right.latencyMs ?? Number.MAX_SAFE_INTEGER)
      )
    })
  }, [currentEndpoint, probes])
}

function findPreferredEndpoint(probes: ApiEndpointProbe[]) {
  return probes
    .filter(probe => probe.available && probe.latencyMs != null)
    .sort((left, right) => left.latencyMs! - right.latencyMs!)[0]
}

function EndpointDisplay({
  endpoint,
  probe,
  isDiscovering,
  compact = false
}: {
  endpoint: string
  probe: ApiEndpointProbe | undefined
  isDiscovering: boolean
  compact?: boolean
}) {
  return (
    <span className="flex w-full min-w-0 items-center justify-between gap-2">
      <span className="truncate">{formatEndpoint(endpoint)}</span>
      <EndpointHealthBadge probe={probe} isDiscovering={isDiscovering} compact={compact} />
    </span>
  )
}

function EndpointHealthBadge({
  probe,
  isDiscovering,
  compact = false
}: {
  probe: ApiEndpointProbe | undefined
  isDiscovering: boolean
  compact?: boolean
}) {
  if (isDiscovering && !probe?.latencyMs) {
    return (
      <span className="inline-flex shrink-0 items-center gap-1 text-xs text-muted-foreground">
        <LoaderCircleIcon className="size-3 animate-spin" />
        {compact ? null : '探测中'}
      </span>
    )
  }

  if (probe && !probe.available && probe.error) {
    return (
      <span className="inline-flex shrink-0 items-center gap-1 text-xs text-destructive">
        <XCircleIcon className="size-3" />
        {compact ? '失败' : '不可用'}
      </span>
    )
  }

  if (!probe || probe.latencyMs == null) {
    return (
      <span className="inline-flex shrink-0 items-center gap-1 text-xs text-muted-foreground">
        <XCircleIcon className="size-3" />
        {compact ? '未测' : '未测试'}
      </span>
    )
  }

  return (
    <span
      className={cn(
        'inline-flex shrink-0 items-center gap-1 text-xs',
        latencyTone(probe.latencyMs)
      )}
    >
      <CheckCircle2Icon className="size-3" />
      {probe.latencyMs} ms
    </span>
  )
}

function latencyTone(latencyMs: number) {
  if (latencyMs <= 500) {
    return 'text-emerald-600 dark:text-emerald-400'
  }

  if (latencyMs <= 1500) {
    return 'text-amber-600 dark:text-amber-400'
  }

  return 'text-orange-600 dark:text-orange-400'
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

function formatCacheLimit(limitMb: number) {
  return limitMb >= 1024 ? `${limitMb / 1024} GB` : `${limitMb} MB`
}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) {
    return '0 B'
  }

  const units = ['B', 'KB', 'MB', 'GB']
  let value = bytes
  let unitIndex = 0

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }

  return `${value >= 10 || unitIndex === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unitIndex]}`
}

function SectionTitle({ icon, title }: { icon: ReactNode; title: string }) {
  return (
    <div className="flex items-center gap-2 text-sm font-semibold">
      <span className="text-muted-foreground">{icon}</span>
      {title}
    </div>
  )
}

function SettingRow({
  title,
  description,
  children
}: {
  title: string
  description: string
  children: ReactNode
}) {
  return (
    <div className="flex items-center justify-between gap-6">
      <div className="min-w-0 space-y-1">
        <div className="text-sm font-medium">{title}</div>
        <div className="text-xs leading-5 text-muted-foreground">{description}</div>
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

function formatEndpoint(endpoint: string) {
  return endpoint.replace(/^https?:\/\//, '')
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
