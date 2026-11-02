import { useQuery } from '@tanstack/react-query'

import { listDownloadTasks, type DownloadTask } from '@/lib/api/download'
import { queryKeys } from '@/lib/query-keys'

export function isDownloadActive(task: Pick<DownloadTask, 'status'>) {
  return task.status === 'running' || task.status === 'queued'
}

export function useDownloadTasksQuery({ poll = false } = {}) {
  return useQuery({
    queryKey: queryKeys.downloadTasks(),
    queryFn: ({ signal }) => listDownloadTasks(signal),
    refetchInterval: poll
      ? query => (query.state.data?.tasks.some(isDownloadActive) ? 1000 : false)
      : false,
    staleTime: 10_000,
    refetchOnMount: true,
    refetchOnWindowFocus: true
  })
}
