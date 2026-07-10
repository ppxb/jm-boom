import { createFileRoute } from '@tanstack/react-router'
import { Trash2Icon } from 'lucide-react'
import { useMemo } from 'react'
import { toast } from 'sonner'

import { BackTopButton } from '@/components/back-top-button'
import { ComicGrid } from '@/components/comic'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import { useLocalFavoritesStore } from '@/stores/local-favorites-store'

export const Route = createFileRoute('/_app/favorites')({
  component: FavoritesPage
})

function FavoritesPage() {
  const items = useLocalFavoritesStore(state => state.items)
  const clear = useLocalFavoritesStore(state => state.clear)
  const sortedItems = useMemo(
    () => [...items].sort((left, right) => right.updatedAt - left.updatedAt),
    [items]
  )

  function clearFavorites() {
    clear()
    toast.success('本地收藏已清空')
  }

  return (
    <main className="relative min-h-screen bg-background px-4 pt-6 pb-36 text-foreground sm:px-6 sm:pb-28 lg:px-8">
      <div className="mx-auto w-full max-w-6xl space-y-6">
        <PageHeader title="收藏" description="保存在当前浏览器中的漫画">
          <ConfirmDialog
            trigger={
              <Button variant="outline" size="sm" disabled={sortedItems.length === 0}>
                <Trash2Icon className="size-4" />
                清空
              </Button>
            }
            icon={<Trash2Icon className="size-5 text-destructive" />}
            title="清空本地收藏"
            description="这会删除当前浏览器保存的全部收藏记录，操作后无法恢复。"
            confirmText="确认清空"
            variant="destructive"
            onConfirm={clearFavorites}
          />
        </PageHeader>

        {sortedItems.length === 0 ? (
          <EmptyState emoji="(･o･;)" title="暂无本地收藏" />
        ) : (
          <ComicGrid items={sortedItems} />
        )}
      </div>
      <BackTopButton />
    </main>
  )
}
