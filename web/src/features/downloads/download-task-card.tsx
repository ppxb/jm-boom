import { useNavigate } from '@tanstack/react-router'
import {
  BanIcon,
  CheckCircle2Icon,
  ClockIcon,
  LoaderCircleIcon,
  PauseIcon,
  PlayIcon,
  RotateCcwIcon
} from 'lucide-react'

import { ComicCard } from '@/components/comic'
import { Button } from '@/components/ui/button'
import type { DownloadTask, DownloadTaskStatus } from '@/lib/api/download'
import { formatBytes, formatDuration } from '@/lib/format'

export function DownloadTaskCard({
  task,
  isCancelling,
  isPausing,
  isResuming,
  selectable,
  selected,
  onCancel,
  onPause,
  onResume,
  onSelect
}: {
  task: DownloadTask
  isCancelling: boolean
  isPausing: boolean
  isResuming: boolean
  selectable: boolean
  selected: boolean
  onCancel: () => void
  onPause: () => void
  onResume: () => void
  onSelect: (taskId: string, checked: boolean) => void
}) {
  const navigate = useNavigate()
  const progress = task.totalPages > 0 ? task.completedPages / task.totalPages : 0
  const progressPercent = Math.min(100, Math.round(progress * 100))
  const firstChapter = task.chapters[0]
  const canOpen = task.status === 'completed' && firstChapter !== undefined

  return (
    <ComicCard
      comic={{
        id: task.albumId,
        title: task.comicTitle,
        image: `/api/covers/${encodeURIComponent(task.albumId)}`
      }}
      ratio="square"
      showIdBadge
      progress={task.status === 'completed' ? undefined : progress}
      selectable={selectable}
      selected={selected}
      onSelect={(_, checked) => onSelect(task.taskId, checked)}
      onOpen={
        canOpen && !selectable
          ? () => {
              void navigate({
                to: '/reader/$comicId',
                params: { comicId: firstChapter.chapterId },
                search: { albumId: task.albumId, page: 1 }
              })
            }
          : undefined
      }
      coverOverlay={
        !selectable ? (
          <DownloadCoverOverlay
            task={task}
            progressPercent={progressPercent}
            isCancelling={isCancelling}
            isPausing={isPausing}
            isResuming={isResuming}
            onCancel={onCancel}
            onPause={onPause}
            onResume={onResume}
          />
        ) : undefined
      }
      metadata={
        <>
          <p className="line-clamp-1 text-xs text-muted-foreground">{formatChapterSummary(task)}</p>
          {task.status !== 'completed' ? (
            <p
              className={
                task.error
                  ? 'line-clamp-1 text-xs text-destructive'
                  : 'text-xs text-muted-foreground'
              }
            >
              {task.error || formatTaskMeta(task, progressPercent)}
            </p>
          ) : null}
        </>
      }
    />
  )
}

function DownloadCoverOverlay({
  task,
  progressPercent,
  isCancelling,
  isPausing,
  isResuming,
  onCancel,
  onPause,
  onResume
}: {
  task: DownloadTask
  progressPercent: number
  isCancelling: boolean
  isPausing: boolean
  isResuming: boolean
  onCancel: () => void
  onPause: () => void
  onResume: () => void
}) {
  const canPause = task.status === 'queued' || task.status === 'running'
  const canResume = task.status === 'paused' || task.status === 'failed'
  const canCancel =
    task.status === 'queued' || task.status === 'running' || task.status === 'paused'
  const hasActions = canPause || canResume || canCancel

  return (
    <>
      <div className="absolute top-2 right-2 z-40">
        <StatusBadge status={task.status} progressPercent={progressPercent} />
      </div>
      {hasActions ? (
        <div className="pointer-events-none absolute inset-0 z-20 bg-gradient-to-t from-black/75 via-transparent to-black/25 opacity-100 transition-opacity sm:bg-black/50 sm:opacity-0 sm:group-focus-within:opacity-100 sm:group-hover:opacity-100">
          <div
            className="pointer-events-auto absolute inset-0 flex flex-wrap items-center justify-center gap-2"
            onClick={event => event.stopPropagation()}
            onKeyDown={event => event.stopPropagation()}
          >
            {canPause ? (
              <ActionButton label="暂停下载" disabled={isPausing} onClick={onPause}>
                <PauseIcon className="size-4" />
              </ActionButton>
            ) : null}
            {canResume ? (
              <ActionButton
                label={task.status === 'failed' ? '重新下载' : '恢复下载'}
                disabled={isResuming}
                onClick={onResume}
              >
                {task.status === 'failed' ? (
                  <RotateCcwIcon className="size-4" />
                ) : (
                  <PlayIcon className="size-4" />
                )}
              </ActionButton>
            ) : null}
            {canCancel ? (
              <ActionButton label="取消下载" disabled={isCancelling} onClick={onCancel}>
                <BanIcon className="size-4" />
              </ActionButton>
            ) : null}
          </div>
        </div>
      ) : null}
    </>
  )
}

function ActionButton({
  label,
  disabled,
  onClick,
  children
}: {
  label: string
  disabled: boolean
  onClick: () => void
  children: React.ReactNode
}) {
  return (
    <Button
      type="button"
      variant="secondary"
      size="icon-sm"
      title={label}
      aria-label={label}
      disabled={disabled}
      onClick={onClick}
    >
      {children}
    </Button>
  )
}

function StatusBadge({
  status,
  progressPercent
}: {
  status: DownloadTaskStatus
  progressPercent: number
}) {
  return (
    <div className="flex items-center gap-1 rounded-full border border-white/15 bg-black/55 px-2 py-1 text-[10px] text-white shadow-sm backdrop-blur">
      <StatusIcon status={status} />
      <span>{status === 'running' ? `${progressPercent}%` : statusLabel(status)}</span>
    </div>
  )
}

function StatusIcon({ status }: { status: DownloadTaskStatus }) {
  if (status === 'completed') return <CheckCircle2Icon className="size-3 text-emerald-400" />
  if (status === 'running') return <LoaderCircleIcon className="size-3 animate-spin" />
  if (status === 'queued') return <ClockIcon className="size-3" />
  if (status === 'paused') return <PauseIcon className="size-3" />
  return <BanIcon className="size-3" />
}

function statusLabel(status: DownloadTaskStatus) {
  if (status === 'completed') return '已完成'
  if (status === 'queued') return '排队中'
  if (status === 'paused') return '已暂停'
  if (status === 'cancelled') return '已取消'
  if (status === 'failed') return '失败'
  return '下载中'
}

function formatChapterSummary(task: DownloadTask) {
  return task.chapters.length > 1
    ? `${task.chapters.length} 个章节`
    : task.chapters[0]?.title || '正文'
}

function formatTaskMeta(task: DownloadTask, progressPercent: number) {
  const speed = `${formatBytes(task.speedBytesPerSecond)}/S`
  if (task.status === 'running') {
    return task.etaSeconds && task.etaSeconds > 0
      ? `${progressPercent}% · ${speed} · 剩余约 ${formatDuration(task.etaSeconds)}`
      : `${progressPercent}% · ${speed}`
  }
  if (task.status === 'queued') return '等待下载'
  if (task.status === 'paused') return `${progressPercent}% · 已暂停`
  if (task.status === 'cancelled') return '下载已取消'
  return '下载失败'
}
