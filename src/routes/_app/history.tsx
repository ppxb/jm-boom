import { createFileRoute } from '@tanstack/react-router'
import { CheckSquareIcon, XIcon } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'

import { BackTopButton } from '@/components/back-top-button'
import { ComicCard } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import { ClearHistoryDialog, DeleteSelectedHistoryDialog } from '@/features/history/history-dialogs'
import { formatDate } from '@/lib/format'
import { useReadingHistoryStore } from '@/stores/reading-history-store'

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
    toast.success(`已删除 ${comicIds.length} 条历史观看记录`)
  }

  return (
    <main className="relative min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-6xl space-y-6 p-[32px_32px_16px_96px]">
        <PageHeader title="历史观看" description="本地保存的历史观看进度">
          {isSelecting ? (
            <>
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
                  toast.success('历史观看记录已清除')
                }}
              />
            </>
          )}
        </PageHeader>

        {sortedItems.length === 0 ? (
          <EmptyState emoji="(˙ᯅ˙)" title="暂无历史观看记录" />
        ) : (
          <div className="grid grid-cols-4 gap-6">
            {sortedItems.map(item => {
              const progress = (item.pageIndex + 1) / item.pageCount
              return (
                <ComicCard
                  key={item.comicId}
                  comic={{
                    id: item.comicId,
                    title: item.title,
                    image: item.coverUrl?.trim() ?? ''
                  }}
                  ratio="square"
                  showIdBadge
                  progress={progress}
                  selectable={isSelecting}
                  selected={selectedComicIds.has(item.comicId)}
                  onSelect={toggleItemSelection}
                  linkProps={
                    !isSelecting
                      ? {
                          to: '/reader/$comicId',
                          params: { comicId: item.chapterId },
                          search: {
                            title: item.title,
                            chapter: item.chapterTitle,
                            albumId: item.albumId,
                            fromDetail: '',
                            pageIndex: String(item.pageIndex),
                            nextId: '',
                            nextChapter: ''
                          }
                        }
                      : undefined
                  }
                  metadata={
                    <>
                      <p className="line-clamp-1 text-xs text-muted-foreground">{item.chapterTitle}</p>
                      <p className="text-xs text-muted-foreground">
                        {item.pageIndex + 1}/{item.pageCount} • {formatDate(item.updatedAt)}
                      </p>
                    </>
                  }
                />
              )
            })}
          </div>
        )}
      </div>
      <BackTopButton />
    </main>
  )
}
