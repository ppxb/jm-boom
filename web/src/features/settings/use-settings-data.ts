import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import {
  clearServerCache,
  getEndpointState,
  getSystemInfo,
  refreshApiEndpoints,
  setApiEndpoint
} from '@/lib/api/setting'
import { queryKeys } from '@/lib/query-keys'

export function useSettingsData() {
  const queryClient = useQueryClient()

  const endpointState = useQuery({
    queryKey: queryKeys.apiEndpointDiscovery(),
    queryFn: getEndpointState,
    staleTime: 30 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })
  const systemInfo = useQuery({
    queryKey: queryKeys.settingsSystem(),
    queryFn: getSystemInfo,
    staleTime: 15 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })

  const refreshEndpoints = useMutation({
    mutationFn: refreshApiEndpoints,
    onSuccess: data => {
      queryClient.setQueryData(queryKeys.apiEndpointDiscovery(), data)
      toast.success('接口测速已完成')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  const changeEndpoint = useMutation({
    mutationFn: setApiEndpoint,
    onSuccess: data => {
      queryClient.setQueryData(queryKeys.apiEndpointDiscovery(), data)
      toast.success(data.mode === 'auto' ? '已启用自动优选' : '接口已切换')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  const clearCache = useMutation({
    mutationFn: clearServerCache,
    onSuccess: data => {
      queryClient.setQueryData(queryKeys.settingsSystem(), data)
      toast.success('服务端缓存已清除')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  return { endpointState, systemInfo, refreshEndpoints, changeEndpoint, clearCache }
}
