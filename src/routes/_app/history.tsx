import { createFileRoute, Link } from '@tanstack/react-router'
import { CheckSquareIcon, Clock3Icon, Trash2Icon, XIcon } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'

import { PageHeader } from '@/components/page-header'
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
import { Checkbox } from '@/components/ui/checkbox'
import { ComicCover } from '@/components/comic-cover'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { cn } from '@/lib/utils'
import { useReadingHistoryStore, type ReadingHistoryItem } from '@/stores/reading-history-store'

export const Route = createFileRoute('/_app/history')({
  component: HistoryPage
})

function HistoryPage() {
  const items = useReadingHistoryStore(state => state.items)
  const removeMany = useReadingHistoryStore(state => state.removeMany)
  const clear = useReadingHistoryStore(state => state.clear)
  const [isSelecting, setIsSelecting] = useState(false)
  const [selectedComicIds, setSelectedComicIds] = useState<Set<string>>(() => new Set())
  const sortedItems = useMemo(
    () => [...items].sort((left, right) => right.updatedAt - left.updatedAt),
    [items]
  )
  const selectedCount = selectedComicIds.size
  const allSelected = sortedItems.length > 0 && selectedCount === sortedItems.length

  useEffect(() => {
    const availableComicIds = new Set(items.map(item => item.comicId))

    setSelectedComicIds(current => {
      const next = new Set([...current].filter(comicId => availableComicIds.has(comicId)))

      return next.size === current.size ? current : next
    })

    if (items.length === 0) {
      setIsSelecting(false)
    }
  }, [items])

  function toggleSelectionMode(nextSelecting: boolean) {
    setIsSelecting(nextSelecting)

    if (!nextSelecting) {
      setSelectedComicIds(new Set())
    }
  }

  function toggleSelectAll() {
    setSelectedComicIds(allSelected ? new Set() : new Set(sortedItems.map(item => item.comicId)))
  }

  function toggleItemSelection(comicId: string, checked: boolean) {
    setSelectedComicIds(current => {
      const next = new Set(current)

      if (checked) {
        next.add(comicId)
      } else {
        next.delete(comicId)
      }

      return next
    })
  }

  function deleteSelectedHistory() {
    const comicIds = [...selectedComicIds]

    if (comicIds.length === 0) {
      return
    }

    removeMany(comicIds)
    setSelectedComicIds(new Set())
    setIsSelecting(false)
    toast.success(`已删除 ${comicIds.length} 条阅读记录`)
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-6xl space-y-6 p-[32px_32px_16px_96px]">
        <PageHeader title="历史观看" desc="本地保存的阅读进度">
          {isSelecting ? (
            <>
              <span className="text-sm text-muted-foreground">
                {selectedCount > 0 ? `已选 ${selectedCount} 条` : '选择要删除的记录'}
              </span>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={sortedItems.length === 0}
                onClick={toggleSelectAll}
              >
                <CheckSquareIcon className="size-4" />
                {allSelected ? '取消全选' : '全选'}
              </Button>
              <DeleteSelectedHistoryDialog
                count={selectedCount}
                disabled={selectedCount === 0}
                onConfirm={deleteSelectedHistory}
              />
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => toggleSelectionMode(false)}
              >
                <XIcon className="size-4" />
                退出
              </Button>
            </>
          ) : (
            <>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={sortedItems.length === 0}
                onClick={() => toggleSelectionMode(true)}
              >
                <CheckSquareIcon className="size-4" />
                选择
              </Button>
              <ClearHistoryDialog
                disabled={sortedItems.length === 0}
                onConfirm={() => {
                  clear()
                  toast.success('阅读记录已清除')
                }}
              />
            </>
          )}
        </PageHeader>

        {sortedItems.length === 0 ? (
          <EmptyState />
        ) : (
          <div className="grid grid-cols-4 gap-6">
            {sortedItems.map(item => (
              <HistoryCard
                key={item.comicId}
                item={item}
                isSelecting={isSelecting}
                isSelected={selectedComicIds.has(item.comicId)}
                onSelectionChange={toggleItemSelection}
              />
            ))}
          </div>
        )}
      </div>
    </main>
  )
}

function DeleteSelectedHistoryDialog({
  count,
  disabled,
  onConfirm
}: {
  count: number
  disabled: boolean
  onConfirm: () => void
}) {
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button type="button" variant="destructive" size="sm" disabled={disabled}>
          <Trash2Icon className="size-4" />
          删除选中
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <div className="flex items-start gap-3 py-1">
          <div className="flex size-10 shrink-0 items-center justify-center rounded-full bg-destructive/10 dark:bg-destructive/10">
            <Trash2Icon className="size-5 text-destructive" />
          </div>
          <div className="flex flex-col justify-center gap-1">
            <AlertDialogTitle className="text-sm font-semibold">删除阅读记录</AlertDialogTitle>
            <AlertDialogDescription className="text-sm text-muted-foreground">
              这会删除选中的 {count} 条本地阅读进度，删除后无法恢复。
            </AlertDialogDescription>
          </div>
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel>取消</AlertDialogCancel>
          <AlertDialogAction variant="destructive" onClick={onConfirm}>
            确认删除
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
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

function HistoryCard({
  item,
  isSelecting,
  isSelected,
  onSelectionChange
}: {
  item: ReadingHistoryItem
  isSelecting: boolean
  isSelected: boolean
  onSelectionChange: (comicId: string, checked: boolean) => void
}) {
  const coverSrc = item.coverUrl?.trim() ?? ''
  const progress = item.pageCount > 0 ? ((item.pageIndex + 1) / item.pageCount) * 100 : 0
  const title = item.title || `JM ${item.comicId}`
  const card = (
    <Card
      size="sm"
      role={isSelecting ? 'button' : undefined}
      aria-pressed={isSelecting ? isSelected : undefined}
      tabIndex={isSelecting ? 0 : undefined}
      className={cn(
        'gap-0 overflow-hidden py-0 transition-shadow hover:cursor-pointer hover:shadow-xl'
      )}
      onClick={isSelecting ? () => onSelectionChange(item.comicId, !isSelected) : undefined}
      onKeyDown={
        isSelecting
          ? event => {
              if (event.key !== 'Enter' && event.key !== ' ') {
                return
              }

              event.preventDefault()
              onSelectionChange(item.comicId, !isSelected)
            }
          : undefined
      }
    >
      <div className="relative">
        {isSelecting ? (
          <div className="absolute top-4 right-4 z-30">
            <Checkbox
              checked={isSelected}
              className="data-checked:border-green-500 data-checked:bg-green-500 dark:data-checked:border-green-500 dark:data-checked:bg-green-500"
              onClick={event => event.stopPropagation()}
              onKeyDown={event => event.stopPropagation()}
              onCheckedChange={checked => onSelectionChange(item.comicId, checked === true)}
            />
          </div>
        ) : null}
        <ComicCover id={item.comicId} title={title} image={coverSrc} ratio="square" showIdBadge />
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
  )

  if (isSelecting) {
    return <div className="block">{card}</div>
  }

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
      {card}
    </Link>
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
