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
import type { ApiEndpointProbe, EndpointState } from '@/lib/api/setting'
import { cn } from '@/lib/utils'
import { formatEndpoint } from './use-endpoint-options'
import { SettingRow, SettingsSection } from './shared'

const AUTO_ENDPOINT_VALUE = '__auto__'

export function ApiEndpointSection({
  state,
  isLoading,
  isRefreshing,
  isChanging,
  onEndpointChange,
  onRefresh
}: {
  state: EndpointState | undefined
  isLoading: boolean
  isRefreshing: boolean
  isChanging: boolean
  onEndpointChange: (endpoint: string | null) => void
  onRefresh: () => void
}) {
  const value =
    state?.mode === 'manual' && state.selectedEndpoint
      ? state.selectedEndpoint
      : AUTO_ENDPOINT_VALUE
  const activeProbe = state?.endpoints.find(item => item.endpoint === state.currentEndpoint)

  return (
    <SettingsSection icon={<NetworkIcon className="size-4" />} title="网络">
      <SettingRow title="上游接口" description="可选择最快的接口，或使用自动优选">
        <div className="flex w-full items-center gap-2 sm:w-auto">
          <Select
            value={value}
            disabled={isLoading || isChanging}
            onValueChange={next => onEndpointChange(next === AUTO_ENDPOINT_VALUE ? null : next)}
          >
            <SelectTrigger className="min-w-0 flex-1 sm:w-72">
              <SelectValue>
                {value === AUTO_ENDPOINT_VALUE ? (
                  <span className="flex items-center gap-2">
                    <span>自动优选</span>
                    {state?.currentEndpoint ? (
                      <span className="hidden text-xs text-muted-foreground sm:inline">
                        {formatEndpoint(state.currentEndpoint)}
                      </span>
                    ) : null}
                  </span>
                ) : (
                  <EndpointDisplay endpoint={value} probe={activeProbe} />
                )}
              </SelectValue>
            </SelectTrigger>
            <SelectContent position="popper" align="end">
              <SelectGroup>
                <SelectItem value={AUTO_ENDPOINT_VALUE}>自动优选</SelectItem>
                {state?.endpoints.map(option => (
                  <SelectItem key={option.endpoint} value={option.endpoint}>
                    <EndpointDisplay endpoint={option.endpoint} probe={option} />
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <Button
            type="button"
            variant="outline"
            size="icon"
            disabled={isLoading || isRefreshing}
            onClick={onRefresh}
            aria-label="重新测速"
          >
            <RefreshCwIcon className={cn('size-4', isRefreshing && 'animate-spin')} />
          </Button>
        </div>
      </SettingRow>
    </SettingsSection>
  )
}

function EndpointDisplay({ endpoint, probe }: { endpoint: string; probe?: ApiEndpointProbe }) {
  return (
    <span className="flex w-full min-w-0 items-center justify-between gap-2">
      <span className="truncate">{formatEndpoint(endpoint)}</span>
      {probe?.available && probe.latencyMs != null ? (
        <span
          className={cn('inline-flex items-center gap-1 text-xs', latencyTone(probe.latencyMs))}
        >
          <CheckCircle2Icon className="size-3" />
          {probe.latencyMs} ms
        </span>
      ) : probe?.error ? (
        <span className="inline-flex items-center gap-1 text-xs text-destructive">
          <XCircleIcon className="size-3" />
          不可用
        </span>
      ) : (
        <span className="inline-flex items-center gap-1 text-xs text-muted-foreground">
          <LoaderCircleIcon className="size-3" />
          未测试
        </span>
      )}
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
