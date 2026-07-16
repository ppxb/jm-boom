import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import {
  clearServerCache,
  getSystemInfo
} from '@/lib/api/setting'
import { queryKeys } from '@/lib/query-keys'

export function useSettingsData() {
  const queryClient = useQueryClient()

  const systemInfo = useQuery({
    queryKey: queryKeys.settingsSystem(),
    queryFn: getSystemInfo,
    staleTime: 15 * 1000,
    retry: false,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false
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

  return {
    systemInfo,
    clearCache
  }
}
