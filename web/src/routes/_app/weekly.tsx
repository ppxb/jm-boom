import { useQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { CalendarDaysIcon } from 'lucide-react'
import { useEffect, useMemo } from 'react'

import { BackTopButton } from '@/components/back-top-button'
import { ComicGrid, ComicGridSkeleton } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
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
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { getWeekFilters, getWeekItems } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { parseStringSearch } from '@/lib/utils'

type WeeklySearch = {
  categoryId: string
  typeId: string
}

export const Route = createFileRoute('/_app/weekly')({
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
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-6xl space-y-6 p-[32px_32px_16px_96px]">
        <PageBackButton />
        <PageHeader title="每周推荐" description="为你精选的本周热门作品" />

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
            <div className="mb-4 flex justify-between gap-3">
              {types.length > 0 ? (
                <Tabs value={selectedTypeId} onValueChange={updateTypeId}>
                  <TabsList>
                    {types.map(type => (
                      <TabsTrigger key={type.id} value={type.id} className="min-w-16">
                        {type.title}
                      </TabsTrigger>
                    ))}
                  </TabsList>
                </Tabs>
              ) : (
                <div className="h-9 w-64 animate-pulse rounded-md bg-muted" />
              )}

              {categories.length > 0 ? (
                <Select value={selectedCategoryId} onValueChange={updateCategoryId}>
                  <SelectTrigger>
                    <CalendarDaysIcon className="size-4 text-muted-foreground" />
                    <SelectValue placeholder="选择期数" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {categories.map(category => (
                        <SelectItem key={category.id} value={category.id}>
                          {category.label}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              ) : (
                <div className="h-9 w-full animate-pulse rounded-md bg-muted lg:w-[320px]" />
              )}
            </div>

            <section>
              {items.isError ? (
                <EmptyState
                  emoji="Ò︵Ó"
                  title="数据加载失败"
                  actions={
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => items.refetch()}
                    >
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
            </section>
          </>
        )}
      </div>
      <BackTopButton />
    </main>
  )
}
