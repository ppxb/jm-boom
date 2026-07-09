import {
  BanIcon,
  CheckCircle2Icon,
  ClockIcon,
  FolderOpenIcon,
  LoaderCircleIcon,
  PauseIcon,
  PlayIcon,
  RotateCcwIcon
} from 'lucide-react'

import { OverflowTooltip } from '@/components/overflow-tooltip'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import type { DownloadTask, DownloadTaskStatus } from '@/lib/api/download'
import { formatBytes, formatDuration } from '@/lib/format'
import { cn } from '@/lib/utils'
import { DeleteTaskDialog } from './delete-task-dialog'

export function DownloadTaskCard({
  task,
  isCancelling,
  isPausing,
  isResuming,
  isRemoving,
  isOpening,
  onCancel,
  onPause,
  onResume,
  onRemove,
  onOpen
}: {
  task: DownloadTask
  isCancelling: boolean
  isPausing: boolean
  isResuming: boolean
  isRemoving: boolean
  isOpening: boolean
  onCancel: () => void
  onPause: () => void
  onResume: () => void
  onRemove: () => void
  onOpen: () => void
}) {
  const progress = task.totalPages > 0 ? task.completedPages / task.totalPages : 0
  const progressPercent = Math.min(100, Math.round(progress * 100))
  const canPause = task.status === 'queued' || task.status === 'running'
  const canResume = task.status === 'paused'
  const canRetry = task.status === 'failed'
  const canCancel =
    task.status === 'queued' || task.status === 'running' || task.status === 'paused'
  const canRemove = task.status !== 'running'

  return (
    <Card>
      <CardContent className="space-y-4 p-5">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0 flex-1">
            <div className="flex min-w-0 items-center gap-2">
              <StatusIcon status={task.status} />
              <OverflowTooltip asChild content={task.comicTitle}>
                <h2 className="min-w-0 flex-1 truncate text-base font-medium">
                  {task.comicTitle}
                </h2>
              </OverflowTooltip>
            </div>
            <div className="mt-1 text-xs text-muted-foreground">{formatChapterSummary(task)}</div>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <Button variant="outline" size="sm" disabled={isOpening} onClick={onOpen}>
              <FolderOpenIcon className="size-4" />
              目录
            </Button>
            {canPause ? (
              <Button variant="outline" size="sm" disabled={isPausing} onClick={onPause}>
                <PauseIcon className="size-4" />
                暂停
              </Button>
            ) : null}
            {canResume ? (
              <Button variant="outline" size="sm" disabled={isResuming} onClick={onResume}>
                <PlayIcon className="size-4" />
                恢复
              </Button>
            ) : null}
            {canRetry ? (
              <Button variant="outline" size="sm" disabled={isResuming} onClick={onResume}>
                <RotateCcwIcon className="size-4" />
                重试
              </Button>
            ) : null}
            {canCancel ? (
              <Button variant="outline" size="sm" disabled={isCancelling} onClick={onCancel}>
                <BanIcon className="size-4" />
                取消
              </Button>
            ) : null}
            {canRemove ? (
              <DeleteTaskDialog
                comicTitle={task.comicTitle}
                disabled={isRemoving}
                onConfirm={onRemove}
              />
            ) : null}
          </div>
        </div>

        <div className="space-y-2">
          <Progress value={progressPercent} className="h-1.5" />
          <div className="flex items-center justify-between text-xs text-muted-foreground">
            <span>{formatProgressLabel(task, progressPercent)}</span>
            <span>{formatTaskMeta(task)}</span>
          </div>
        </div>

        {task.error ? <div className="text-xs text-destructive">{task.error}</div> : null}
      </CardContent>
    </Card>
  )
}

function formatChapterSummary(task: DownloadTask) {
  const chapterCount = task.chapters.length

  return chapterCount > 1 ? `${chapterCount} 个章节` : task.chapters[0]?.title || '正文'
}

function formatProgressLabel(task: DownloadTask, progressPercent: number) {
  return task.totalPages > 0 ? `${progressPercent}%` : '准备中'
}

function StatusIcon({ status }: { status: DownloadTaskStatus }) {
  if (status === 'completed') {
    return <CheckCircle2Icon className="size-4 text-emerald-600" />
  }

  if (status === 'running') {
    return <LoaderCircleIcon className="size-4 animate-spin text-primary" />
  }

  if (status === 'queued') {
    return <ClockIcon className="size-4 text-muted-foreground" />
  }

  if (status === 'paused') {
    return <PauseIcon className="size-4 text-muted-foreground" />
  }

  return (
    <BanIcon
      className={cn('size-4', status === 'failed' ? 'text-destructive' : 'text-muted-foreground')}
    />
  )
}

function formatTaskMeta(task: DownloadTask) {
  const speed = `${formatBytes(task.speedBytesPerSecond)}/S`

  if (task.status === 'running') {
    return task.etaSeconds && task.etaSeconds > 0
      ? `${speed} · 剩余约 ${formatDuration(task.etaSeconds)}`
      : `${speed} · 正在下载`
  }

  if (task.status === 'queued') return `${speed} · 等待中`
  if (task.status === 'paused') return '已暂停'
  if (task.status === 'completed') return '已完成'
  if (task.status === 'cancelled') return '已取消'
  return '失败'
}
