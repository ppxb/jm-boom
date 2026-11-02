import { useQueryClient } from '@tanstack/react-query'
import { ClockIcon, LoaderCircleIcon } from 'lucide-react'
import { useEffect, useRef } from 'react'
import { toast } from 'sonner'

import { Progress } from '@/components/ui/progress'
import type { DownloadTask } from '@/lib/api/download'
import { formatBytes } from '@/lib/format'
import { queryKeys } from '@/lib/query-keys'
import { isDownloadActive, useDownloadTasksQuery } from './use-download-tasks-query'

type NotificationState = { fingerprint: string; active: boolean; status: DownloadTask['status'] }

const submittedDownloads = new Set<string>()

export function markDownloadSubmitted(taskId: string) {
  submittedDownloads.add(taskId)
}

export function DownloadNotifications() {
  const tasks = useDownloadTasksQuery({ poll: true })
  const queryClient = useQueryClient()
  const previous = useRef(new Map<string, NotificationState>())
  const dismissed = useRef(new Set<string>())
  const initialized = useRef(false)

  useEffect(
    () => () => {
      for (const id of previous.current.keys()) toast.dismiss(id)
      previous.current.clear()
      dismissed.current.clear()
      initialized.current = false
    },
    []
  )

  useEffect(() => {
    if (!tasks.data) return
    const next = new Map<string, NotificationState>()
    let completed = false
    for (const task of tasks.data.tasks) {
      const id = `download:${task.taskId}`
      const active = isDownloadActive(task)
      const fingerprint = `${task.status}|${task.completedPages}|${task.totalPages}|${task.speedBytesPerSecond}|${task.error}|${active ? tasks.isError : task.updatedAt}`
      const old = previous.current.get(id)
      const submitted = submittedDownloads.delete(task.taskId)
      next.set(id, { fingerprint, active, status: task.status })
      if (old?.fingerprint === fingerprint && !submitted) continue
      if (active) {
        if (old && !old.active) dismissed.current.delete(id)
        if (!dismissed.current.has(id)) {
          notifyDownload(task, tasks.isError, () => dismissed.current.add(id))
        }
      } else if (submitted || old?.active || initialized.current) {
        notifyDownload(task, false)
        dismissed.current.delete(id)
      }
      if (task.status === 'completed' && old?.status !== 'completed') completed = true
    }
    for (const id of previous.current.keys()) {
      if (!next.has(id)) {
        toast.dismiss(id)
        dismissed.current.delete(id)
      }
    }
    previous.current = next
    initialized.current = true
    if (completed) void queryClient.invalidateQueries({ queryKey: queryKeys.downloadedChapters() })
  }, [tasks.data, tasks.isError, queryClient])

  return null
}

function notifyDownload(task: DownloadTask, waiting: boolean, onDismiss?: () => void) {
  const active = isDownloadActive(task)
  const options = {
    id: `download:${task.taskId}`,
    duration: active ? Infinity : 8000,
    closeButton: true,
    icon: undefined,
    onDismiss
  }
  if (active) {
    const percent =
      task.totalPages > 0
        ? Math.min(100, Math.round((task.completedPages / task.totalPages) * 100))
        : 0
    toast.info(task.comicTitle, {
      ...options,
      icon:
        task.status === 'queued' && !waiting ? (
          <ClockIcon className="size-4" />
        ) : (
          <LoaderCircleIcon className="size-4 animate-spin" />
        ),
      description: waiting ? (
        '正在重新连接下载服务…'
      ) : (
        <div className="space-y-2">
          <p>
            {task.status === 'queued'
              ? '排队中'
              : `${percent}% · ${task.completedPages} / ${task.totalPages} 页 · ${formatBytes(task.speedBytesPerSecond)}/s`}
          </p>
          {task.status === 'running' && task.totalPages > 0 ? (
            <Progress value={percent} className="h-1.5" />
          ) : null}
        </div>
      )
    })
  } else if (task.status === 'completed') {
    toast.success(task.comicTitle, { ...options, description: '下载完成' })
  } else if (task.status === 'failed') {
    toast.error(task.comicTitle, { ...options, description: task.error || '下载失败，请重试' })
  } else {
    toast.info(task.comicTitle, {
      ...options,
      description: task.status === 'paused' ? '已暂停下载' : '已取消下载'
    })
  }
}
