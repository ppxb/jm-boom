import { useQuery, useQueryClient } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { SearchIcon } from 'lucide-react'
import { FormEvent, useEffect, useState } from 'react'

import { ComicCard, ComicRail, ComicRailItem } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { ListPagination } from '@/components/list-pagination'
import { PageBackButton } from '@/components/page-back-button'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { InputGroup, InputGroupAddon, InputGroupInput } from '@/components/ui/input-group'
import { Skeleton } from '@/components/ui/skeleton'
import { useSourceCatalog } from '@/features/source/use-source-catalog'
import {
  mapSourceManga,
  searchInstalledSources,
  type InstalledSource,
  type SourceSearchGroup
} from '@/lib/api/source'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'

type SearchPageSearch = {
  q?: string
  page?: number
}

export const Route = createFileRoute('/_app/explore/search')({
  validateSearch: validateSearchParams,
  component: SearchPage
})

function SearchPage() {
  const navigate = useNavigate({ from: Route.fullPath })
  const queryClient = useQueryClient()
  const search = Route.useSearch()
  const keyword = search.q ?? ''
  const page = search.page ?? 1
  const [draftKeyword, setDraftKeyword] = useState(keyword)
  const { sources, isLoading: isSourceListLoading } = useSourceCatalog({
    includeCatalog: false
  })
  const sourceVersions = sources.map(source => `${source.info.id}@${source.info.version}`)

  useEffect(() => {
    setDraftKeyword(keyword)
  }, [keyword])

  const query = useQuery({
    queryKey: queryKeys.sourceSearch(keyword, page, sourceVersions),
    queryFn: () => searchInstalledSources(sources, keyword, page),
    enabled: keyword.length > 0 && sources.length > 0,
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    void navigate({
      replace: true,
      search: createSearchParams({ q: draftKeyword })
    })
  }

  function updatePage(nextPage: number) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: createSearchParams({ q: keyword, page: nextPage })
    })
  }

  function openManga(source: InstalledSource, manga: SourceSearchGroup['entries'][number]) {
    queryClient.setQueryData(queryKeys.sourceManga(source.info.id, manga.key), manga)
    void navigate({
      to: '/comic/$comicId',
      params: { comicId: manga.key },
      search: { sourceId: source.info.id }
    })
  }

  return (
    <section className="space-y-6">
      <PageBackButton
        onClick={() => {
          void navigate({ to: '/explore', replace: true })
        }}
      />

      <form className="w-full max-w-xl" onSubmit={submitSearch}>
        <InputGroup>
          <InputGroupAddon>
            <SearchIcon className="size-4" />
          </InputGroupAddon>
          <InputGroupInput
            type="text"
            role="searchbox"
            inputMode="search"
            value={draftKeyword}
            onChange={event => setDraftKeyword(event.target.value)}
            placeholder="搜索漫画名称、作者或编号"
            aria-label="搜索关键词"
            enterKeyHint="search"
          />
        </InputGroup>
      </form>

      <SearchContent
        keyword={keyword}
        page={page}
        sources={sources}
        groups={query.data ?? []}
        isSourceListLoading={isSourceListLoading}
        isLoading={query.isLoading}
        isFetching={query.isFetching}
        isError={query.isError}
        onRetry={() => query.refetch()}
        onOpenSettings={() => void navigate({ to: '/settings' })}
        onOpenManga={openManga}
        onPageChange={updatePage}
      />
    </section>
  )
}

function SearchContent({
  keyword,
  page,
  sources,
  groups,
  isSourceListLoading,
  isLoading,
  isFetching,
  isError,
  onRetry,
  onOpenSettings,
  onOpenManga,
  onPageChange
}: {
  keyword: string
  page: number
  sources: InstalledSource[]
  groups: SourceSearchGroup[]
  isSourceListLoading: boolean
  isLoading: boolean
  isFetching: boolean
  isError: boolean
  onRetry: () => void
  onOpenSettings: () => void
  onOpenManga: (
    source: InstalledSource,
    manga: SourceSearchGroup['entries'][number]
  ) => void
  onPageChange: (page: number) => void
}) {
  if (keyword.length === 0) return null

  if (isSourceListLoading) {
    return <SearchSectionsSkeleton sources={[]} />
  }

  if (sources.length === 0) {
    return (
      <EmptyState
        emoji="(･o･;)"
        title="还没有安装漫画源，请先在设置中从可信目录安装"
        actions={
          <Button type="button" variant="outline" size="sm" onClick={onOpenSettings}>
            前往设置
          </Button>
        }
      />
    )
  }

  if (isError) {
    return (
      <EmptyState
        emoji="Ò︵Ó"
        title="搜索请求失败"
        actions={
          <Button type="button" variant="outline" size="sm" onClick={onRetry}>
            重试
          </Button>
        }
      />
    )
  }

  if (isLoading) {
    return <SearchSectionsSkeleton sources={sources} />
  }

  const hasNextPage = groups.some(group => group.hasNextPage)

  return (
    <div className="space-y-10">
      {groups.map(group => (
        <SourceSearchSection
          key={group.source.info.id}
          group={group}
          onRetry={onRetry}
          onOpenManga={onOpenManga}
        />
      ))}

      {page > 1 || hasNextPage ? (
        <ListPagination
          page={page}
          hasMore={hasNextPage}
          disabled={isFetching}
          onPageChange={onPageChange}
        />
      ) : null}
    </div>
  )
}

function SourceSearchSection({
  group,
  onRetry,
  onOpenManga
}: {
  group: SourceSearchGroup
  onRetry: () => void
  onOpenManga: (
    source: InstalledSource,
    manga: SourceSearchGroup['entries'][number]
  ) => void
}) {
  return (
    <section className="min-w-0 space-y-4">
      <div className="flex items-end justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <div className="flex items-center gap-2">
            <h2 className="truncate text-xl font-semibold">{group.source.info.name}</h2>
            <Badge variant="outline">v{group.source.info.version}</Badge>
          </div>
          <p className="truncate text-xs text-muted-foreground">{group.source.info.id}</p>
        </div>
        {group.entries.length > 0 ? (
          <span className="shrink-0 text-xs text-muted-foreground">
            {group.entries.length} 项
          </span>
        ) : null}
      </div>

      {group.error ? (
        <div className="flex items-center justify-between gap-3 rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-3">
          <p className="line-clamp-2 text-sm text-destructive">{group.error}</p>
          <Button type="button" variant="outline" size="sm" onClick={onRetry}>
            重试
          </Button>
        </div>
      ) : group.entries.length === 0 ? (
        <p className="rounded-xl border border-dashed px-4 py-6 text-center text-sm text-muted-foreground">
          这个源没有匹配结果
        </p>
      ) : (
        <ComicRail>
          {group.entries.map(manga => {
            const comic = mapSourceManga(manga)
            return (
              <ComicRailItem key={`${group.source.info.id}:${manga.key}`}>
                <ComicCard
                  comic={comic}
                  ratio="square"
                  onOpen={() => onOpenManga(group.source, manga)}
                  metadata={
                    <p className="line-clamp-1 text-xs text-muted-foreground">
                      {comic.author || '未知作者'}
                    </p>
                  }
                />
              </ComicRailItem>
            )
          })}
        </ComicRail>
      )}
    </section>
  )
}

function SearchSectionsSkeleton({ sources }: { sources: InstalledSource[] }) {
  const sections = sources.length > 0 ? sources : [null, null]
  return (
    <div className="space-y-10">
      {sections.map((source, sectionIndex) => (
        <section key={source?.info.id ?? sectionIndex} className="space-y-4">
          <Skeleton className="h-7 w-36" />
          <ComicRail>
            {Array.from({ length: 4 }).map((_, itemIndex) => (
              <ComicRailItem key={itemIndex}>
                <Card size="sm" className="gap-0 overflow-hidden py-0">
                  <Skeleton className="aspect-square w-full rounded-none" />
                  <CardContent className="space-y-1.5 p-3">
                    <Skeleton className="h-4 w-full" />
                    <Skeleton className="h-3 w-2/3" />
                  </CardContent>
                </Card>
              </ComicRailItem>
            ))}
          </ComicRail>
        </section>
      ))}
    </div>
  )
}

function validateSearchParams(search: Record<string, unknown>): SearchPageSearch {
  return createSearchParams({
    q: typeof search.q === 'string' ? search.q : '',
    page: parseOptionalPage(search.page)
  })
}

function createSearchParams({ q, page = 1 }: { q: string; page?: number }): SearchPageSearch {
  const keyword = q.trim()
  if (keyword.length === 0) return {}
  return {
    q: keyword,
    ...(page > 1 ? { page } : {})
  }
}

function parseOptionalPage(value: unknown) {
  const page = Number(value)
  return Number.isSafeInteger(page) && page > 1 ? page : undefined
}
