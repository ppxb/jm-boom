import { useEffect, useRef, useState } from 'react'

import { useSettingsStore } from '@/stores/settings-store'
import { findPreferredEndpoint } from './use-endpoint-options'

export function useAutoEndpointSelection(
  endpointDiscoveryData: any,
  endpointDiscoveryDataUpdatedAt: number
) {
  const api = useSettingsStore(state => state.api)
  const setApi = useSettingsStore(state => state.setApi)
  const apiRef = useRef(api)
  const lastPreferredDiscoveryAtRef = useRef(0)
  const [isRefreshingEndpoints, setIsRefreshingEndpoints] = useState(false)

  useEffect(() => {
    apiRef.current = api
  }, [api])

  useEffect(() => {
    if (
      !endpointDiscoveryData ||
      endpointDiscoveryDataUpdatedAt === 0 ||
      lastPreferredDiscoveryAtRef.current === endpointDiscoveryDataUpdatedAt
    ) {
      return
    }

    lastPreferredDiscoveryAtRef.current = endpointDiscoveryDataUpdatedAt

    const preferredEndpoint = findPreferredEndpoint(endpointDiscoveryData)

    if (preferredEndpoint && apiRef.current !== preferredEndpoint.endpoint) {
      setApi(preferredEndpoint.endpoint)
    }

    setIsRefreshingEndpoints(false)
  }, [endpointDiscoveryData, endpointDiscoveryDataUpdatedAt, setApi])

  return { isRefreshingEndpoints, setIsRefreshingEndpoints }
}
