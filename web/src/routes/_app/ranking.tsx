import { useQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { BarChart3Icon, ListFilterIcon } from 'lucide-react'

import { BackTopButton } from '@/components/back-top-button'
import { ComicGrid, ComicGridSkeleton } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { ListPagination } from '@/components/list-pagination'
import { PageBackButton } from '@/components/page-back-button'
import { Button } from '@/components/ui/button'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import { getHomeSectionList } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import {
  defaultRankingCategory,
  rankingCategoryApiValue,
  rankingCategoryOptions,
  RANKING_ORDER_OPTIONS
} from '@/lib/filters'
import { parsePositivePage, parseStringSearch } from '@/lib/utils'

type RankingSearch = {
  page: number
  category: string
  order: string
}

export const Route = createFileRoute('/_app/ranking')({
  validateSearch: (search: Record<string, unknown>): RankingSearch => ({
    page: parsePositivePage(search.page),
    category: parseRankingCategory(search.category),
    order: parseRankingOrder(search.order)
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
        order: parseRankingOrder(value)
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
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-6xl space-y-6 p-[32px_32px_16px_96px]">
        <PageBackButton />
        <PageHeader title="排行榜" description="按分类和热度浏览作品" />

        <div className="mb-4 flex items-center justify-end gap-3">
          <Select value={search.order} onValueChange={updateOrder}>
            <SelectTrigger>
              <ListFilterIcon className="size-4 text-muted-foreground" />
              <SelectValue placeholder="选择排序" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {RANKING_ORDER_OPTIONS.map(option => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>

          <Select value={search.category} onValueChange={updateCategory}>
            <SelectTrigger>
              <BarChart3Icon className="size-4 text-muted-foreground" />
              <SelectValue placeholder="选择分类" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {categories.map(option => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
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
      </div>
      <BackTopButton />
    </main>
  )
}

function parseRankingCategory(value: unknown) {
  const category = parseStringSearch(value, defaultRankingCategory())

  return rankingCategoryOptions().some(option => option.value === category)
    ? category
    : defaultRankingCategory()
}

function parseRankingOrder(value: unknown) {
  const order = parseStringSearch(value, 'new')

  return RANKING_ORDER_OPTIONS.some(option => option.value === order) ? order : 'new'
}
