import { useQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { BarChart3Icon, ListFilterIcon } from 'lucide-react'

import { ComicGrid, ComicGridSkeleton } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { FilterSelect } from '@/components/filter-select'
import { ListPagination } from '@/components/list-pagination'
import { Button } from '@/components/ui/button'
import { getHomeSectionList } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import {
  rankingCategoryApiValue,
  rankingCategoryOptions,
  RANKING_ORDER_OPTIONS
} from '@/lib/filters'
import { parsePositivePage } from '@/lib/utils'
import { parseListOrder, parseRankingCategory } from '@/features/section-list/section-utils'

type RankingSearch = {
  page: number
  category: string
  order: string
}

export const Route = createFileRoute('/_app/explore/ranking')({
  validateSearch: (search: Record<string, unknown>): RankingSearch => ({
    page: parsePositivePage(search.page),
    category: parseRankingCategory(search.category),
    order: parseListOrder(search.order)
  }),
  component: RankingPage
})

function RankingPage() {
  const navigate = useNavigate({ from: Route.fullPath })
  const search = Route.useSearch()
  const categories = rankingCategoryOptions()

  const query = useQuery({
    queryKey: queryKeys.ranking(search.page, search.category, search.order),
    queryFn: () =>
      getHomeSectionList({
        mode: 'ranking',
        page: search.page,
        sectionTitle: '排行榜',
        category: rankingCategoryApiValue(search.category),
        order: search.order
      }),
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const items = query.data?.items ?? []

  function updateCategory(value: string) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        ...search,
        page: 1,
        category: parseRankingCategory(value)
      }
    })
  }

  function updateOrder(value: string) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        ...search,
        page: 1,
        order: parseListOrder(value)
      }
    })
  }

  function updatePage(page: number) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        ...search,
        page
      }
    })
  }

  return (
    <section className="space-y-6">
      <div className="flex items-center justify-end gap-2">
        <FilterSelect
          value={search.order}
          options={RANKING_ORDER_OPTIONS}
          placeholder="选择排序"
          icon={<ListFilterIcon className="size-4 text-muted-foreground" />}
          onValueChange={updateOrder}
        />
        <FilterSelect
          value={search.category}
          options={categories}
          placeholder="选择分类"
          icon={<BarChart3Icon className="size-4 text-muted-foreground" />}
          onValueChange={updateCategory}
        />
      </div>

      {query.isError ? (
        <EmptyState
          emoji="Ò︵Ó"
          title="数据加载失败"
          actions={
            <Button type="button" variant="outline" size="sm" onClick={() => query.refetch()}>
              重试
            </Button>
          }
        />
      ) : query.isLoading ? (
        <ComicGridSkeleton count={12} />
      ) : items.length === 0 ? (
        <EmptyState emoji="(･o･;)" title="暂无排行内容" />
      ) : (
        <>
          <ComicGrid items={items} />
          <ListPagination
            page={search.page}
            hasMore={query.data?.hasMore ?? false}
            disabled={query.isFetching}
            onPageChange={updatePage}
          />
        </>
      )}
    </section>
  )
}
