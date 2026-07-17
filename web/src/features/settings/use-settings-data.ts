import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import {
  clearAccount,
  clearServerCache,
  getAccountState,
  getEndpointState,
  getSystemInfo,
  refreshApiEndpoints,
  setApiEndpoint,
  updateAccount
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
    refetchOnMount: 'always',
    refetchOnWindowFocus: false
  })
  const account = useQuery({
    queryKey: queryKeys.settingsAccount(),
    queryFn: getAccountState,
    staleTime: 10_000,
    refetchInterval: query => {
      const state = query.state.data
      return state?.loginStatus === 'loggingIn' || state?.signInStatus === 'signingIn'
        ? 1000
        : false
    },
    refetchOnWindowFocus: true
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

  const saveAccount = useMutation({
    mutationFn: updateAccount,
    onSuccess: data => {
      queryClient.setQueryData(queryKeys.settingsAccount(), data)
      toast.success('禁漫天堂账号设置已保存')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '禁漫天堂账号登录失败')
    }
  })

  const removeAccount = useMutation({
    mutationFn: clearAccount,
    onSuccess: data => {
      queryClient.setQueryData(queryKeys.settingsAccount(), data)
      toast.success('禁漫天堂账号已退出登录')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '禁漫天堂账号退出登录失败')
    }
  })

  return {
    endpointState,
    systemInfo,
    account,
    refreshEndpoints,
    changeEndpoint,
    clearCache,
    saveAccount,
    removeAccount
  }
}
