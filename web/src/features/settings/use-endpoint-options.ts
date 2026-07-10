import { useMemo } from 'react'

import type { ApiEndpointProbe } from '@/lib/api/setting'
import { FALLBACK_API_ENDPOINTS } from '@/stores/settings-store'

export function useEndpointOptions(
  currentEndpoint: string,
  probes: ApiEndpointProbe[] | undefined
) {
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

export function findPreferredEndpoint(probes: ApiEndpointProbe[]) {
  return probes
    .filter(probe => probe.available && probe.latencyMs != null)
    .sort((left, right) => left.latencyMs! - right.latencyMs!)[0]
}

export function formatEndpoint(endpoint: string) {
  return endpoint.replace(/^https?:\/\//, '')
}
