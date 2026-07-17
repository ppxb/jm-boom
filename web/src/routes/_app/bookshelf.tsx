import { createFileRoute } from '@tanstack/react-router'
import { Trash2Icon } from 'lucide-react'
import { useMemo } from 'react'
import { toast } from 'sonner'

import { AppPage } from '@/components/app-page'
import { ComicCard } from '@/components/comic'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { SelectionActions } from '@/components/selection-actions'
import { Button } from '@/components/ui/button'
import { useHistorySelection } from '@/features/history/use-history-selection'
import { formatDate } from '@/lib/format'
import { useReadingHistoryStore } from '@/stores/reading-history-store'

export const Route = createFileRoute('/_app/bookshelf')({
  component: BookshelfPage
})

function BookshelfPage() {
  const items = useReadingHistoryStore(state => state.items)
  const removeMany = useReadingHistoryStore(state => state.removeMany)
  const clear = useReadingHistoryStore(state => state.clear)

  const sortedItems = useMemo(
    () => [...items].sort((left, right) => right.lastReadAt - left.lastReadAt),
    [items]
  )

  const selection = useHistorySelection(sortedItems)

  function deleteSelectedHistory() {
    const comicIds = [...selection.selectedComicIds]

    if (comicIds.length === 0) {
      return
    }

    removeMany(comicIds)
    selection.toggleSelectionMode(false)
    toast.success(`已删除 ${comicIds.length} 条历史观看记录`)
  }

  function clearAllHistory() {
    clear()
    toast.success('历史观看记录已清除')
  }

  return (
    <AppPage>
      <PageHeader title="书架" description="继续阅读或管理漫画">
        <SelectionActions
          isSelecting={selection.isSelecting}
          allSelected={selection.allSelected}
          selectedCount={selection.selectedCount}
          disabled={sortedItems.length === 0}
          enterLabel="选择清除"
          dialogTitle="清除历史观看记录"
          dialogDescription={`这会清除选中的 ${selection.selectedCount} 条本地观看记录，清除后无法恢复。`}
          confirmText="确认清除"
          onEnter={() => selection.toggleSelectionMode(true)}
          onExit={() => selection.toggleSelectionMode(false)}
          onToggleAll={selection.toggleSelectAll}
          onDeleteSelected={deleteSelectedHistory}
          idleActions={
            <ConfirmDialog
              trigger={
                <Button
                  type="button"
                  variant="destructive"
                  size="sm"
                  disabled={sortedItems.length === 0}
                >
                  <Trash2Icon className="size-4" />
                  清除全部
                </Button>
              }
              icon={<Trash2Icon className="size-5 text-destructive" />}
              title="清除历史观看记录"
              description="这会删除本地保存的全部阅读记录，清除后无法恢复。"
              confirmText="确认清除"
              variant="destructive"
              onConfirm={clearAllHistory}
            />
          }
        />
      </PageHeader>

      {sortedItems.length === 0 ? (
        <EmptyState className="min-h-0 flex-1" emoji="(˙ᯅ˙)" title="书架还是空的" />
      ) : (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4 lg:grid-cols-4 lg:gap-6">
          {sortedItems.map(item => {
            const progress = (item.pageIndex + 1) / item.pageCount
            return (
              <ComicCard
                key={item.id}
                comic={{
                  id: item.id,
                  title: item.title,
                  image: item.image.trim()
                }}
                ratio="square"
                showIdBadge
                progress={progress}
                selectable={selection.isSelecting}
                selected={selection.selectedComicIds.has(item.id)}
                onSelect={selection.toggleSelectItem}
                linkProps={
                  !selection.isSelecting
                    ? {
                        to: '/reader/$comicId',
                        params: { comicId: item.chapterId },
                        search: {
                          albumId: item.id,
                          page: item.pageIndex + 1
                        }
                      }
                    : undefined
                }
                metadata={
                  <>
                    <p className="line-clamp-1 text-xs text-muted-foreground">
                      {item.chapterTitle}
                    </p>
                    <p className="text-xs text-muted-foreground">{formatDate(item.lastReadAt)}</p>
                  </>
                }
              />
            )
          })}
        </div>
      )}
    </AppPage>
  )
}
