import { createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { LoaderCircleIcon, Trash2Icon } from 'lucide-react'
import { useState } from 'react'
import { toast } from 'sonner'

import { AppPage } from '@/components/app-page'
import { ComicGrid } from '@/components/comic'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { EmptyState } from '@/components/empty-state'
import { ListPagination } from '@/components/list-pagination'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import type { ComicStateResult } from '@/lib/api/comic'
import { clearFavorites, listFavorites } from '@/lib/api/favorite'
import { UI } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'

export const Route = createFileRoute('/_app/favorites')({
  component: FavoritesPage
})

function FavoritesPage() {
  const queryClient = useQueryClient()
  const [page, setPage] = useState(1)
  const { data, isLoading, isError, isFetching, refetch } = useQuery({
    queryKey: queryKeys.favorites(page),
    queryFn: () => listFavorites(page)
  })
  const { mutate: clear, isPending: isClearing } = useMutation({
    mutationFn: clearFavorites,
    onSuccess: () => {
      setPage(1)
      queryClient.setQueriesData({ queryKey: queryKeys.favorites() }, { items: [], total: 0 })
      queryClient.setQueriesData<ComicStateResult>({ queryKey: ['jm-comic-state'] }, current =>
        current ? { ...current, isFavorite: false } : current
      )
      toast.success('收藏已清空')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '清空收藏失败')
    }
  })
  const items = data?.items ?? []
  const total = data?.total ?? 0
  const pageCount = Math.ceil(total / UI.COLLECTION_PAGE_SIZE)

  return (
    <AppPage>
      <PageHeader title="收藏" description="服务端收藏的漫画">
        <ConfirmDialog
          trigger={
            <Button variant="destructive" size="sm" disabled={total === 0 || isClearing}>
              <Trash2Icon className="size-4" />
              清空收藏
            </Button>
          }
          icon={<Trash2Icon className="size-5 text-destructive" />}
          title="清空服务端收藏"
          description="这会删除当前服务端中所有设备共享的收藏记录，操作后无法恢复。"
          confirmText="确认清空"
          variant="destructive"
          loading={isClearing}
          onConfirm={() => clear()}
        />
      </PageHeader>

      {isLoading ? (
        <div className="flex flex-1 items-center justify-center">
          <LoaderCircleIcon className="size-6 animate-spin text-muted-foreground" />
        </div>
      ) : isError ? (
        <EmptyState
          className="min-h-0 flex-1"
          emoji="Ò︵Ó"
          title="收藏加载失败"
          actions={
            <Button variant="outline" size="sm" onClick={() => refetch()}>
              重试
            </Button>
          }
        />
      ) : items.length === 0 ? (
        <EmptyState className="min-h-0 flex-1" emoji="(･o･;)" title="暂无收藏" />
      ) : (
        <ComicGrid items={items} />
      )}

      {pageCount > 1 ? (
        <ListPagination
          page={page}
          hasMore={page < pageCount}
          disabled={isFetching || isClearing}
          onPageChange={setPage}
        />
      ) : null}
    </AppPage>
  )
}
