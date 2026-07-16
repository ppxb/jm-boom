import { useQuery, useQueryClient } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect } from 'react'

import { ComicCard, ComicGridSkeleton } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { ListPagination } from '@/components/list-pagination'
import { PageBackButton } from '@/components/page-back-button'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { useSourceCatalog } from '@/features/source/use-source-catalog'
import {
  getSourceListing,
  mapSourceManga,
  type InstalledSource,
  type SourceManga
} from '@/lib/api/source'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { parsePositivePage, parseStringSearch } from '@/lib/utils'

type SourceListingSearch = {
  sourceId: string
  listingId: string
  page: number
}

export const Route = createFileRoute('/_app/explore/list')({
  validateSearch: (search: Record<string, unknown>): SourceListingSearch => ({
    sourceId: parseStringSearch(search.sourceId),
    listingId: parseStringSearch(search.listingId),
    page: parsePositivePage(search.page)
  }),
  component: SourceListingPage
})

function SourceListingPage() {
  const navigate = useNavigate({ from: Route.fullPath })
  const queryClient = useQueryClient()
  const search = Route.useSearch()
  const { sources, isLoading: isSourceListLoading } = useSourceCatalog({
    includeCatalog: false
  })
  const source = sources.find(item => item.info.id === search.sourceId) ?? null
  const listings = source?.listings ?? []
  const listing =
    listings.find(item => item.id === search.listingId) ?? listings[0] ?? null

  useEffect(() => {
    if (!source || !listing || listing.id === search.listingId) return
    void navigate({
      replace: true,
      resetScroll: false,
      search: {
        sourceId: source.info.id,
        listingId: listing.id,
        page: 1
      }
    })
  }, [listing, navigate, search.listingId, source])

  const query = useQuery({
    queryKey: queryKeys.sourceListing(
      source?.info.id ?? search.sourceId,
      listing?.id ?? search.listingId,
      search.page
    ),
    queryFn: () => getSourceListing(source!.info.id, listing!, search.page),
    enabled: source != null && listing != null,
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })

  function changeListing(listingId: string) {
    if (!source) return
    void navigate({
      replace: true,
      resetScroll: false,
      search: { sourceId: source.info.id, listingId, page: 1 }
    })
  }

  function changePage(page: number) {
    if (!source || !listing) return
    void navigate({
      replace: true,
      resetScroll: false,
      search: { sourceId: source.info.id, listingId: listing.id, page }
    })
  }

  function openManga(manga: SourceManga) {
    if (!source) return
    queryClient.setQueryData(queryKeys.sourceManga(source.info.id, manga.key), manga)
    void navigate({
      to: '/comic/$comicId',
      params: { comicId: manga.key },
      search: { sourceId: source.info.id }
    })
  }

  if (isSourceListLoading) {
    return <ComicGridSkeleton count={12} />
  }

  if (!source) {
    return (
      <section className="space-y-6">
        <PageBackButton />
        <EmptyState emoji="(･o･;)" title="漫画源未安装或已被移除" />
      </section>
    )
  }

  if (!listing) {
    return (
      <section className="space-y-6">
        <PageBackButton />
        <PageHeader title={source.info.name} description="该源暂未提供探索列表" />
        <EmptyState emoji="(･o･;)" title="暂无可浏览分类" />
      </section>
    )
  }

  const entries = query.data?.entries ?? []

  return (
    <section className="space-y-6">
      <PageBackButton />
      <PageHeader title={source.info.name} description={listing.name} />

      {listings.length > 1 ? (
        <Tabs value={listing.id} onValueChange={changeListing}>
          <TabsList className="max-w-full justify-start overflow-x-auto">
            {listings.map(item => (
              <TabsTrigger key={item.id} value={item.id}>
                {item.name}
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      ) : null}

      {query.isError ? (
        <EmptyState
          emoji="Ò︵Ó"
          title="漫画源列表加载失败"
          actions={
            <Button type="button" variant="outline" size="sm" onClick={() => query.refetch()}>
              重试
            </Button>
          }
        />
      ) : query.isLoading ? (
        <ComicGridSkeleton count={12} />
      ) : entries.length === 0 ? (
        <EmptyState emoji="(･o･;)" title="这个分类暂无内容" />
      ) : (
        <>
          <SourceMangaGrid source={source} entries={entries} onOpenManga={openManga} />
          <ListPagination
            page={search.page}
            hasMore={query.data?.hasNextPage ?? false}
            disabled={query.isFetching}
            onPageChange={changePage}
          />
        </>
      )}
    </section>
  )
}

function SourceMangaGrid({
  source,
  entries,
  onOpenManga
}: {
  source: InstalledSource
  entries: SourceManga[]
  onOpenManga: (manga: SourceManga) => void
}) {
  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4 lg:grid-cols-4 lg:gap-6">
      {entries.map(manga => {
        const comic = mapSourceManga(manga)
        return (
          <ComicCard
            key={`${source.info.id}:${manga.key}`}
            comic={comic}
            ratio="square"
            onOpen={() => onOpenManga(manga)}
            metadata={
              <p className="line-clamp-1 text-xs text-muted-foreground">
                {comic.author || '未知作者'}
              </p>
            }
          />
        )
      })}
    </div>
  )
}
