import { createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Trash2Icon } from 'lucide-react'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'

import { AppPage } from '@/components/app-page'
import { ComicCard } from '@/components/comic'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { EmptyState } from '@/components/empty-state'
import { ListPagination } from '@/components/list-pagination'
import { PageHeader } from '@/components/page-header'
import { SelectionActions } from '@/components/selection-actions'
import { Button } from '@/components/ui/button'
import { useHistorySelection } from '@/features/history/use-history-selection'
import type { ComicStateResult } from '@/lib/api/comic'
import {
  clearReadingHistory,
  listReadingHistory,
  removeReadingHistory,
  type ReadingHistoryListResult
} from '@/lib/api/history'
import { UI } from '@/lib/constants'
import { formatDate } from '@/lib/format'
import { queryKeys } from '@/lib/query-keys'

export const Route = createFileRoute('/_app/bookshelf')({
  component: BookshelfPage
})

function BookshelfPage() {
  const queryClient = useQueryClient()
  const [page, setPage] = useState(1)
  const history = useQuery({
    queryKey: queryKeys.readingHistory(page),
    queryFn: () => listReadingHistory(page),
    staleTime: 10_000,
    refetchOnMount: true,
    refetchOnWindowFocus: true
  })
  const items = history.data?.items ?? []
  const total = history.data?.total ?? 0
  const pageCount = Math.max(1, Math.ceil(total / UI.COLLECTION_PAGE_SIZE))
  const selection = useHistorySelection(items)
  const removeMutation = useMutation({
    mutationFn: removeReadingHistory,
    onSuccess: (_, comicIds) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.readingHistory() })
      for (const comicId of comicIds) {
        queryClient.setQueryData<ComicStateResult>(queryKeys.comicState(comicId), current =>
          current ? { ...current, history: null } : current
        )
      }
      selection.toggleSelectionMode(false)
      toast.success('已删除选中的历史观看记录')
    },
    onError: showHistoryError
  })
  const clearMutation = useMutation({
    mutationFn: clearReadingHistory,
    onSuccess: () => {
      queryClient.setQueriesData<ReadingHistoryListResult>(
        { queryKey: queryKeys.readingHistory() },
        { items: [], total: 0 }
      )
      queryClient.setQueriesData<ComicStateResult>(
        { queryKey: ['jm-comic-state'] },
        current => (current ? { ...current, history: null } : current)
      )
      toast.success('历史观看记录已清除')
    },
    onError: showHistoryError
  })

  useEffect(() => {
    if (!history.data) {
      return
    }
    setPage(current => Math.min(current, pageCount))
  }, [history.data, pageCount])

  function deleteSelectedHistory() {
    const comicIds = [...selection.selectedComicIds]

    if (comicIds.length === 0) {
      return
    }

    removeMutation.mutate(comicIds)
  }

  return (
    <AppPage>
      <PageHeader title="书架" description="继续阅读或管理漫画">
        <SelectionActions
          isSelecting={selection.isSelecting}
          allSelected={selection.allSelected}
          selectedCount={selection.selectedCount}
          disabled={total === 0}
          loading={removeMutation.isPending || clearMutation.isPending}
          enterLabel="选择清除"
          dialogTitle="清除历史观看记录"
          dialogDescription={`这会清除选中的 ${selection.selectedCount} 条共享观看记录，清除后无法恢复。`}
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
                  disabled={total === 0 || clearMutation.isPending}
                >
                  <Trash2Icon className="size-4" />
                  清除全部
                </Button>
              }
              icon={<Trash2Icon className="size-5 text-destructive" />}
              title="清除历史观看记录"
              description="这会删除当前实例中所有设备共享的阅读记录，清除后无法恢复。"
              confirmText="确认清除"
              variant="destructive"
              loading={clearMutation.isPending}
              onConfirm={() => clearMutation.mutate()}
            />
          }
        />
      </PageHeader>

      {history.isLoading ? (
        <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
          正在读取阅读历史
        </div>
      ) : history.isError ? (
        <EmptyState
          className="min-h-0 flex-1"
          emoji="Ò︵Ó"
          title="阅读历史加载失败"
          actions={
            <Button variant="outline" size="sm" onClick={() => history.refetch()}>
              重试
            </Button>
          }
        />
      ) : items.length === 0 ? (
        <EmptyState className="min-h-0 flex-1" emoji="(˙ᯅ˙)" title="书架还是空的" />
      ) : (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4 lg:grid-cols-4 lg:gap-6">
          {items.map(item => {
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

      {pageCount > 1 ? (
        <ListPagination
          page={page}
          hasMore={page < pageCount}
          disabled={history.isFetching || removeMutation.isPending || clearMutation.isPending}
          onPageChange={setPage}
        />
      ) : null}
    </AppPage>
  )
}

function showHistoryError(error: unknown) {
  toast.error(error instanceof Error ? error.message : '阅读历史操作失败')
}
