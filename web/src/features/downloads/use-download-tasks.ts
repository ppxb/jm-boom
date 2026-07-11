import { useMemo, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import {
  cancelDownloadTask,
  listDownloadTasks,
  pauseDownloadTask,
  removeDownloadTask,
  resumeDownloadTask,
  type DownloadTask,
  type DownloadTaskListResult
} from '@/lib/api/download'
import { queryKeys } from '@/lib/query-keys'

export const DOWNLOAD_FILTERS = [
  { value: 'all', label: '全部' },
  { value: 'active', label: '下载中' },
  { value: 'paused', label: '已暂停' },
  { value: 'completed', label: '已完成' }
] as const

export type DownloadFilter = (typeof DOWNLOAD_FILTERS)[number]['value']

const EMPTY_DOWNLOAD_TASKS: DownloadTask[] = []

export function useDownloadTasks() {
  const [filter, setFilter] = useState<DownloadFilter>('all')
  const tasks = useQuery({
    queryKey: queryKeys.downloadTasks(),
    queryFn: listDownloadTasks,
    refetchInterval: query =>
      hasActiveTasks(query.state.data?.tasks ?? EMPTY_DOWNLOAD_TASKS) ? 1000 : false,
    refetchOnWindowFocus: false
  })
  const cancelTask = useTaskMutation(cancelDownloadTask, '已取消下载任务')
  const pauseTask = useTaskMutation(pauseDownloadTask, '已暂停下载任务')
  const resumeTask = useTaskMutation(resumeDownloadTask, '已加入下载队列')
  const removeTasks = useTaskMutation(removeDownloadTasks, '已删除选中的下载任务和文件')
  const taskList = tasks.data?.tasks ?? EMPTY_DOWNLOAD_TASKS
  const filterCounts = useMemo(() => getFilterCounts(taskList), [taskList])
  const filteredTasks = useMemo(
    () => taskList.filter(task => matchesFilter(task, filter)),
    [filter, taskList]
  )

  return {
    filter,
    setFilter,
    tasks,
    taskList,
    filteredTasks,
    filterCounts,
    cancelTask,
    pauseTask,
    resumeTask,
    removeTasks
  }
}

function useTaskMutation<TVariables>(
  mutationFn: (variables: TVariables) => Promise<DownloadTaskListResult>,
  message: string
) {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn,
    onSuccess: result => {
      queryClient.setQueryData(queryKeys.downloadTasks(), result)
      void queryClient.invalidateQueries({ queryKey: queryKeys.downloadedChapters() })
      toast.success(message)
    },
    onError: showError
  })
}

async function removeDownloadTasks(taskIds: string[]) {
  let result: DownloadTaskListResult = { tasks: [] }
  for (const taskId of taskIds) {
    result = await removeDownloadTask(taskId)
  }
  return result
}

function getFilterCounts(tasks: DownloadTask[]): Record<DownloadFilter, number> {
  return {
    all: tasks.length,
    active: tasks.filter(task => task.status === 'running' || task.status === 'queued').length,
    paused: tasks.filter(task => task.status === 'paused').length,
    completed: tasks.filter(task => task.status === 'completed').length
  }
}

function matchesFilter(task: DownloadTask, filter: DownloadFilter) {
  if (filter === 'active') return task.status === 'running' || task.status === 'queued'
  if (filter === 'paused') return task.status === 'paused'
  if (filter === 'completed') return task.status === 'completed'
  return true
}

function hasActiveTasks(tasks: DownloadTask[]) {
  return tasks.some(task => task.status === 'running' || task.status === 'queued')
}

function showError(error: unknown) {
  toast.error(error instanceof Error ? error.message : String(error))
}
