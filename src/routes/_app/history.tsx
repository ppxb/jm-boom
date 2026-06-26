import { createFileRoute, Link } from '@tanstack/react-router'
import { Clock3Icon, ImageIcon, Trash2Icon } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogTitle,
  AlertDialogTrigger
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { useSettingsStore } from '@/stores/settings-store'
import { useReadingHistoryStore, type ReadingHistoryItem } from '@/stores/reading-history-store'

export const Route = createFileRoute('/_app/history')({
  component: HistoryPage
})

function HistoryPage() {
  const items = useReadingHistoryStore(state => state.items)
  const clear = useReadingHistoryStore(state => state.clear)
  const hideCovers = useSettingsStore(state => state.hideCovers)
  const sortedItems = useMemo(
    () => [...items].sort((left, right) => right.updatedAt - left.updatedAt),
    [items]
  )

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-6xl space-y-6 p-[96px_32px_32px_96px]">
        <header className="flex items-end justify-between gap-4">
          <div>
            <h1 className="text-3xl font-semibold tracking-normal">历史观看</h1>
            <p className="mt-2 text-sm text-muted-foreground">本地保存的阅读进度</p>
          </div>
          <ClearHistoryDialog
            disabled={sortedItems.length === 0}
            onConfirm={() => {
              clear()
              toast.success('阅读记录已清除')
            }}
          />
        </header>

        {sortedItems.length === 0 ? (
          <EmptyState />
        ) : (
          <div className="grid grid-cols-4 gap-6">
            {sortedItems.map(item => (
              <HistoryCard key={item.comicId} item={item} hideCover={hideCovers} />
            ))}
          </div>
        )}
      </div>
    </main>
  )
}

function ClearHistoryDialog({ disabled, onConfirm }: { disabled: boolean; onConfirm: () => void }) {
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button type="button" variant="destructive" size="sm" disabled={disabled}>
          <Trash2Icon className="size-4" />
          清除记录
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <div className="flex items-start gap-3 py-1">
          <div className="flex size-10 shrink-0 items-center justify-center rounded-full bg-destructive/10 dark:bg-destructive/10">
            <Trash2Icon className="size-5 text-destructive" />
          </div>
          <div className="flex flex-col justify-center gap-1">
            <AlertDialogTitle className="text-sm font-semibold">清除阅读记录</AlertDialogTitle>
            <AlertDialogDescription className="text-sm text-muted-foreground">
              这会删除本地保存的全部阅读进度，清除后无法恢复。
            </AlertDialogDescription>
          </div>
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel>取消</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onConfirm}>
            确认清除
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

function HistoryCard({ item, hideCover }: { item: ReadingHistoryItem; hideCover: boolean }) {
  const [hasImageError, setHasImageError] = useState(false)
  const coverSrc = item.coverUrl?.trim() ?? ''
  const shouldShowImage = coverSrc.length > 0 && !hasImageError
  const progress = item.pageCount > 0 ? ((item.pageIndex + 1) / item.pageCount) * 100 : 0
  const title = item.title || `JM ${item.comicId}`

  useEffect(() => {
    setHasImageError(false)
  }, [coverSrc])

  return (
    <Link
      to="/reader/$comicId"
      params={{ comicId: item.chapterId }}
      search={{
        title,
        chapter: item.chapterTitle,
        albumId: item.albumId,
        fromDetail: '',
        pageIndex: String(item.pageIndex),
        nextId: '',
        nextChapter: ''
      }}
      className="block"
    >
      <Card
        size="sm"
        className="gap-0 overflow-hidden py-0 transition-shadow hover:cursor-pointer hover:shadow-xl"
      >
        <div className="relative aspect-square overflow-hidden bg-muted">
          {shouldShowImage ? (
            <img
              src={coverSrc}
              alt={title}
              loading="lazy"
              referrerPolicy="no-referrer"
              className="h-full w-full object-cover"
              onError={() => setHasImageError(true)}
            />
          ) : (
            <div className="flex h-full items-center justify-center bg-muted text-muted-foreground">
              <ImageIcon className="size-6" />
            </div>
          )}
          {hideCover ? <CoverMask /> : null}
          <div className="absolute top-2 left-2 z-20 rounded-full border border-input/80 bg-background/45 px-2 py-1 text-[10px] backdrop-blur">
            JM {item.comicId}
          </div>
          <div className="absolute right-2 bottom-2 left-2 z-20">
            <div className="h-1 overflow-hidden rounded-full bg-black/40">
              <div className="h-full rounded-full bg-primary" style={{ width: `${progress}%` }} />
            </div>
          </div>
        </div>
        <CardContent className="space-y-1.5 p-3">
          <Tooltip>
            <TooltipTrigger asChild>
              <div className="truncate text-sm font-semibold">{title}</div>
            </TooltipTrigger>
            <TooltipContent side="top">{title}</TooltipContent>
          </Tooltip>
          <p className="line-clamp-1 text-xs text-muted-foreground">{item.chapterTitle}</p>
          {item.author ? (
            <p className="line-clamp-1 text-xs text-muted-foreground">{item.author}</p>
          ) : null}
          <p className="text-xs text-muted-foreground">
            {item.pageIndex + 1}/{item.pageCount} • {formatDate(item.updatedAt)}
          </p>
        </CardContent>
      </Card>
    </Link>
  )
}

function CoverMask() {
  return (
    <div className="absolute inset-0 z-10 flex items-center justify-center bg-muted/90 text-muted-foreground backdrop-blur-sm">
      <ImageIcon className="size-6" />
    </div>
  )
}

function EmptyState() {
  return (
    <Card>
      <CardContent className="flex min-h-48 flex-col items-center justify-center gap-3 text-center">
        <Clock3Icon className="size-8 text-muted-foreground" />
        <div className="text-sm text-muted-foreground">暂无阅读记录</div>
      </CardContent>
    </Card>
  )
}

function formatDate(value: number) {
  return new Intl.DateTimeFormat('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  }).format(new Date(value))
}
