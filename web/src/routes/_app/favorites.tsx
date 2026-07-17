import { createFileRoute } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Trash2Icon } from 'lucide-react'
import { toast } from 'sonner'

import { AppPage } from '@/components/app-page'
import { ComicGrid } from '@/components/comic'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import { clearFavorites, listFavorites } from '@/lib/api/favorite'
import { queryKeys } from '@/lib/query-keys'

export const Route = createFileRoute('/_app/favorites')({
  component: FavoritesPage
})

function FavoritesPage() {
  const queryClient = useQueryClient()
  const favorites = useQuery({
    queryKey: queryKeys.favorites(),
    queryFn: listFavorites,
    staleTime: 0,
    refetchOnMount: 'always',
    refetchOnWindowFocus: true
  })
  const clearMutation = useMutation({
    mutationFn: clearFavorites,
    onSuccess: result => {
      queryClient.setQueryData(queryKeys.favorites(), result)
      toast.success('实例收藏已清空')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '清空收藏失败')
    }
  })
  const items = favorites.data?.items ?? []

  return (
    <AppPage>
      <PageHeader title="收藏" description="同一实例中共享的漫画">
        <ConfirmDialog
          trigger={
            <Button
              variant="outline"
              size="sm"
              disabled={items.length === 0 || clearMutation.isPending}
            >
              <Trash2Icon className="size-4" />
              清空
            </Button>
          }
          icon={<Trash2Icon className="size-5 text-destructive" />}
          title="清空实例收藏"
          description="这会删除当前实例中所有设备共享的收藏记录，操作后无法恢复。"
          confirmText="确认清空"
          variant="destructive"
          loading={clearMutation.isPending}
          onConfirm={() => clearMutation.mutate()}
        />
      </PageHeader>

      {favorites.isLoading ? (
        <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
          正在读取收藏
        </div>
      ) : favorites.isError ? (
        <EmptyState
          className="min-h-0 flex-1"
          emoji="Ò︵Ó"
          title="收藏加载失败"
          actions={
            <Button variant="outline" size="sm" onClick={() => favorites.refetch()}>
              重试
            </Button>
          }
        />
      ) : items.length === 0 ? (
        <EmptyState className="min-h-0 flex-1" emoji="(･o･;)" title="暂无收藏" />
      ) : (
        <ComicGrid items={items} />
      )}
    </AppPage>
  )
}
