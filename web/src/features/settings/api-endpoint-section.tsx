import {
  CheckCircle2Icon,
  LoaderCircleIcon,
  NetworkIcon,
  RefreshCwIcon,
  XCircleIcon
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import type { ApiEndpointProbe } from '@/lib/api/setting'
import { cn } from '@/lib/utils'
import { formatEndpoint } from './use-endpoint-options'
import { SettingRow, SettingsSection } from './shared'

export function ApiEndpointSection({
  endpoint,
  endpointOptions,
  isDiscovering,
  isRefreshingEndpoints,
  onEndpointChange,
  onRefresh
}: {
  endpoint: string
  endpointOptions: ApiEndpointProbe[]
  isDiscovering: boolean
  isRefreshingEndpoints: boolean
  onEndpointChange: (endpoint: string) => void
  onRefresh: () => void
}) {
  return (
    <SettingsSection icon={<NetworkIcon className="size-4" />} title="网络">
      <SettingRow title="API 接口" description="测速后自动优选延迟最低的可用接口">
        <div className="flex items-center gap-2">
          <Select value={endpoint} onValueChange={onEndpointChange}>
            <SelectTrigger>
              <SelectValue>
                <EndpointDisplay
                  endpoint={endpoint}
                  probe={endpointOptions.find(option => option.endpoint === endpoint)}
                  isDiscovering={isDiscovering}
                  isRefreshingEndpoints={isRefreshingEndpoints}
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
                      isDiscovering={isDiscovering}
                      isRefreshingEndpoints={isRefreshingEndpoints}
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
            disabled={isDiscovering || isRefreshingEndpoints}
            onClick={onRefresh}
          >
            <RefreshCwIcon
              className={cn('size-4', (isDiscovering || isRefreshingEndpoints) && 'animate-spin')}
            />
          </Button>
        </div>
      </SettingRow>
    </SettingsSection>
  )
}

function EndpointDisplay({
  endpoint,
  probe,
  isDiscovering,
  isRefreshingEndpoints,
  compact = false
}: {
  endpoint: string
  probe: ApiEndpointProbe | undefined
  isDiscovering: boolean
  isRefreshingEndpoints?: boolean
  compact?: boolean
}) {
  return (
    <span className="flex w-full min-w-0 items-center justify-between gap-2">
      <span className="truncate">{formatEndpoint(endpoint)}</span>
      <EndpointHealthBadge
        probe={probe}
        isDiscovering={isDiscovering}
        isRefreshingEndpoints={isRefreshingEndpoints}
        compact={compact}
      />
    </span>
  )
}

function EndpointHealthBadge({
  probe,
  isDiscovering,
  isRefreshingEndpoints = false,
  compact = false
}: {
  probe: ApiEndpointProbe | undefined
  isDiscovering: boolean
  isRefreshingEndpoints?: boolean
  compact?: boolean
}) {
  if (isDiscovering || isRefreshingEndpoints) {
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
