import { LoaderCircleIcon } from 'lucide-react'
import { toast } from 'sonner'
import { create } from 'zustand'

import {
  exportDownloadTask,
  type DownloadExportFormat,
  type DownloadTask
} from '@/lib/api/download'

type ActiveExport = { taskId: string; format: DownloadExportFormat }

export const useDownloadExportState = create<{ active: ActiveExport | null }>(() => ({
  active: null
}))

export async function startDownloadExport(task: DownloadTask, format: DownloadExportFormat) {
  if (useDownloadExportState.getState().active) return
  const controller = new AbortController()
  const id = `download-export:${task.taskId}`
  useDownloadExportState.setState({ active: { taskId: task.taskId, format } })
  toast.info(task.comicTitle, {
    id,
    description: `正在导出 ${format.toUpperCase()}…`,
    duration: Infinity,
    closeButton: true,
    icon: <LoaderCircleIcon className="size-4 animate-spin" />,
    action: {
      label: '取消导出',
      onClick: event => {
        event.preventDefault()
        controller.abort()
      }
    }
  })
  const finished = { id, duration: 8000, closeButton: true, icon: undefined, action: undefined }
  try {
    const file = await exportDownloadTask(task.taskId, format, controller.signal)
    controller.signal.throwIfAborted()
    const url = URL.createObjectURL(file.blob)
    const link = document.createElement('a')
    link.href = url
    link.download = file.filename
    try {
      document.body.append(link)
      link.click()
    } finally {
      link.remove()
      window.setTimeout(() => URL.revokeObjectURL(url), 1000)
    }
    toast.success(`${format.toUpperCase()} 已生成，下载已开始`, {
      ...finished,
      description: link.download
    })
  } catch (error) {
    if (controller.signal.aborted) {
      toast.info('已取消导出', { ...finished, description: task.comicTitle })
    } else {
      toast.error('导出失败', {
        ...finished,
        description: error instanceof Error ? error.message : '请重试'
      })
    }
  } finally {
    useDownloadExportState.setState({ active: null })
  }
}
