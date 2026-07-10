import { useQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { CalendarDaysIcon, TagsIcon } from 'lucide-react'
import { useEffect, useMemo } from 'react'

import { ComicGrid, ComicGridSkeleton } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { FilterSelect } from '@/components/filter-select'
import { Button } from '@/components/ui/button'
import { getWeekFilters, getWeekItems } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { parseStringSearch } from '@/lib/utils'

type WeeklySearch = {
  categoryId: string
  typeId: string
}

export const Route = createFileRoute('/_app/explore/weekly')({
  validateSearch: (search: Record<string, unknown>): WeeklySearch => ({
    categoryId: parseStringSearch(search.categoryId),
    typeId: parseStringSearch(search.typeId)
  }),
  component: WeeklyPage
})

function WeeklyPage() {
  const navigate = useNavigate({ from: Route.fullPath })
  const search = Route.useSearch()

  const filters = useQuery({
    queryKey: queryKeys.weekFilters(),
    queryFn: getWeekFilters,
    staleTime: CACHE.FILTERS_STALE_TIME,
    gcTime: CACHE.FILTERS_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const categories = useMemo(() => filters.data?.categories ?? [], [filters.data?.categories])
  const types = useMemo(() => filters.data?.types ?? [], [filters.data?.types])
  const categoryOptions = useMemo(
    () => categories.map(category => ({ label: category.label, value: category.id })),
    [categories]
  )
  const typeOptions = useMemo(
    () => types.map(type => ({ label: type.title, value: type.id })),
    [types]
  )

  useEffect(() => {
    if (filters.data == null) {
      return
    }

    const nextCategoryId = categories.some(category => category.id === search.categoryId)
      ? search.categoryId
      : (filters.data.defaultCategoryId ?? categories[0]?.id ?? '')
    const nextTypeId = types.some(type => type.id === search.typeId)
      ? search.typeId
      : (filters.data.defaultTypeId ?? types[0]?.id ?? '')

    if (nextCategoryId !== search.categoryId || nextTypeId !== search.typeId) {
      void navigate({
        replace: true,
        resetScroll: false,
        search: {
          ...search,
          categoryId: nextCategoryId,
          typeId: nextTypeId
        }
      })
    }
  }, [categories, filters.data, navigate, search, types])

  const selectedCategoryId =
    search.categoryId || filters.data?.defaultCategoryId || categories[0]?.id || ''
  const selectedTypeId = search.typeId || filters.data?.defaultTypeId || types[0]?.id || ''
  const canLoadItems = selectedCategoryId.length > 0 && selectedTypeId.length > 0

  const items = useQuery({
    queryKey: queryKeys.weekItems(selectedCategoryId, selectedTypeId),
    queryFn: () =>
      getWeekItems({
        categoryId: selectedCategoryId,
        typeId: selectedTypeId
      }),
    enabled: canLoadItems,
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })

  function updateTypeId(typeId: string) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        ...search,
        typeId
      }
    })
  }

  function updateCategoryId(categoryId: string) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        ...search,
        categoryId
      }
    })
  }

  return (
    <section className="space-y-6">
      {filters.isError ? (
        <EmptyState
          emoji="Ò︵Ó"
          title="数据加载失败"
          actions={
            <Button type="button" variant="outline" size="sm" onClick={() => filters.refetch()}>
              重试
            </Button>
          }
        />
      ) : (
        <>
          <div className="flex items-center justify-end gap-2">
            {types.length > 0 ? (
              <FilterSelect
                value={selectedTypeId}
                options={typeOptions}
                placeholder="选择类型"
                icon={<TagsIcon className="size-4 text-muted-foreground" />}
                onValueChange={updateTypeId}
              />
            ) : (
              <div className="h-9 min-w-0 flex-1 animate-pulse rounded-md bg-muted sm:w-40 sm:flex-none" />
            )}

            {categories.length > 0 ? (
              <FilterSelect
                value={selectedCategoryId}
                options={categoryOptions}
                placeholder="选择期数"
                icon={<CalendarDaysIcon className="size-4 text-muted-foreground" />}
                onValueChange={updateCategoryId}
              />
            ) : (
              <div className="h-9 min-w-0 flex-1 animate-pulse rounded-md bg-muted sm:w-48 sm:flex-none" />
            )}
          </div>

          {items.isError ? (
            <EmptyState
              emoji="Ò︵Ó"
              title="数据加载失败"
              actions={
                <Button type="button" variant="outline" size="sm" onClick={() => items.refetch()}>
                  重试
                </Button>
              }
            />
          ) : !canLoadItems || items.isLoading ? (
            <ComicGridSkeleton />
          ) : items.data == null || items.data.items.length === 0 ? (
            <EmptyState emoji="(･o･;)" title="暂无每周推荐" />
          ) : (
            <ComicGrid items={items.data.items} />
          )}
        </>
      )}
    </section>
  )
}
