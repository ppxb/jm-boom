import { useMutation, useQuery } from '@tanstack/react-query'
import { toast } from 'sonner'

import { getSignInData, signIn, type UserProfile } from '@/lib/api/user'
import { queryKeys } from '@/lib/query-keys'

function findTodayRecord(records: Array<{ day: number; signed: boolean }>) {
  const today = new Date().getDate()
  return records.find(record => record.day === today)
}

type UseMeSignInParams = {
  user: UserProfile | null
  endpoint: string | null
}

function signInSuccessMessage(message: string) {
  return /J\s*coin\s*:\s*\d+.*EXP\s*:\s*\d+/i.test(message) ? '签到成功' : message || '签到成功'
}

export function useMeSignIn({ user, endpoint }: UseMeSignInParams) {
  const userId = user?.id
  const signInQuery = useQuery({
    queryKey: queryKeys.signInData(endpoint, userId),
    queryFn: async () => {
      if (!user) {
        throw new Error('请先登录')
      }

      return getSignInData({ userId: user.id, endpoint })
    },
    enabled: userId != null,
    staleTime: 2 * 60 * 1000,
    gcTime: 30 * 60 * 1000,
    refetchOnWindowFocus: false
  })
  const signInMutation = useMutation({
    mutationFn: async () => {
      const dailyId = signInQuery.data?.dailyId

      if (!user || !dailyId) {
        throw new Error('签到信息尚未准备好')
      }

      return signIn({ userId: user.id, dailyId, endpoint })
    },
    onSuccess: result => {
      toast.success(signInSuccessMessage(result.message))
      void signInQuery.refetch()
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })
  return {
    data: signInQuery.data,
    error: signInQuery.error,
    isLoading: signInQuery.isLoading,
    isFetching: signInQuery.isFetching,
    isSigning: signInMutation.isPending,
    refresh: () => {
      void signInQuery.refetch()
    },
    submitSignIn: () => {
      signInMutation.mutate()
    },
    todayRecord: findTodayRecord(signInQuery.data?.records ?? [])
  }
}
