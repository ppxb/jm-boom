import { useQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'

import { ComicGrid, ComicGridSkeleton } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { ListPagination } from '@/components/list-pagination'
import { PageBackButton } from '@/components/page-back-button'
import { Button } from '@/components/ui/button'
import { getHomeSectionList, type HomeSectionListMode } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { rankingCategoryApiValue } from '@/lib/filters'
import { parsePositivePage, parseStringSearch } from '@/lib/utils'
import { SectionFilters } from '@/features/section-list/section-filters'
import {
  isHomeSectionListMode,
  parseListCategory,
  parseListOrder,
  parseListWeek,
  sectionModeDescription,
  sectionModeTitle
} from '@/features/section-list/section-utils'

type HomeSectionListSearch = {
  mode: HomeSectionListMode
  page: number
  sectionId: string
  title: string
  filterValue: string
  rankTag: string
  category: string
  week: string
  order: string
}

export const Route = createFileRoute('/_app/explore/list')({
  validateSearch: (search: Record<string, unknown>): HomeSectionListSearch => {
    const mode = isHomeSectionListMode(search.mode) ? search.mode : 'promote'
    const rankTag = parseStringSearch(search.rankTag)

    return {
      mode,
      page: parsePositivePage(search.page),
      sectionId: parseStringSearch(search.sectionId),
      title: parseStringSearch(search.title),
      filterValue: parseStringSearch(search.filterValue),
      rankTag,
      category: parseListCategory(mode, rankTag, search.category),
      week: parseListWeek(search.week),
      order: parseListOrder(search.order)
    }
  },
  component: HomeSectionListPage
})

function HomeSectionListPage() {
  const navigate = useNavigate({ from: Route.fullPath })
  const search = Route.useSearch()

  const query = useQuery({
    queryKey: queryKeys.homeSectionList(search),
    queryFn: () =>
      getHomeSectionList({
        mode: search.mode,
        page: search.page,
        sectionId: search.sectionId,
        sectionTitle: search.title,
        filterValue: search.filterValue,
        category:
          search.mode === 'ranking'
            ? rankingCategoryApiValue(search.category, search.rankTag)
            : search.mode === 'weekly'
              ? search.category
              : null,
        week: search.mode === 'weekly' ? search.week : null,
        order: search.mode === 'ranking' ? search.order : null
      }),
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const items = query.data?.items ?? []
  const title = query.data?.title || search.title || sectionModeTitle(search.mode)

  function updateCategory(value: string) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        ...search,
        page: 1,
        category: parseListCategory(search.mode, search.rankTag, value)
      }
    })
  }

  function updateWeek(value: string) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        ...search,
        page: 1,
        week: parseListWeek(value)
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
      <PageBackButton />
      <PageHeader title={title} description={sectionModeDescription(search.mode)} />

      <SectionFilters
        mode={search.mode}
        rankTag={search.rankTag}
        category={search.category}
        week={search.week}
        order={search.order}
        onCategoryChange={updateCategory}
        onWeekChange={updateWeek}
        onOrderChange={updateOrder}
      />

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
        <EmptyState emoji="(･o･;)" title="暂无内容" />
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
