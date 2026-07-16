import { useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { ArrowRightIcon } from 'lucide-react'

import { ComicCard, ComicRail, ComicRailItem } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { useSourceCatalog } from '@/features/source/use-source-catalog'
import {
  getSourceListing,
  mapSourceManga,
  type InstalledSource,
  type SourceListing,
  type SourceManga
} from '@/lib/api/source'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { SourcePreviewSkeleton } from './source-preview-skeleton'

type SourcePreview = {
  source: InstalledSource
  listing: SourceListing
  entries: SourceManga[]
  error: string | null
}

export function HomePage() {
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { sources, isLoading: isSourceListLoading } = useSourceCatalog({
    includeCatalog: false
  })
  const previewSources = sources.filter(source => (source.listings?.length ?? 0) > 0)
  const sourceVersions = previewSources.map(
    source => `${source.info.id}@${source.info.version}:${source.listings[0]?.id ?? ''}`
  )
  const previews = useQuery({
    queryKey: queryKeys.sourceListingPreviews(sourceVersions),
    queryFn: () => loadSourcePreviews(previewSources),
    enabled: previewSources.length > 0,
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })

  function openManga(source: InstalledSource, manga: SourceManga) {
    queryClient.setQueryData(queryKeys.sourceManga(source.info.id, manga.key), manga)
    void navigate({
      to: '/comic/$comicId',
      params: { comicId: manga.key },
      search: { sourceId: source.info.id }
    })
  }

  if (isSourceListLoading) {
    return <SourcePreviewSkeleton />
  }

  if (sources.length === 0) {
    return (
      <EmptyState
        emoji="(･o･;)"
        title="还没有安装漫画源"
        actions={
          <Button asChild type="button" variant="outline" size="sm">
            <Link to="/settings">前往设置</Link>
          </Button>
        }
      />
    )
  }

  if (previewSources.length === 0) {
    return <EmptyState emoji="(･o･;)" title="已安装源暂未提供探索列表" />
  }

  return (
    <div className="min-w-0 space-y-10">
      {previews.isLoading ? (
        <SourcePreviewSkeleton />
      ) : (
        (previews.data ?? []).map(preview => (
          <SourcePreviewSection
            key={preview.source.info.id}
            preview={preview}
            onRetry={() => previews.refetch()}
            onOpenManga={openManga}
          />
        ))
      )}
    </div>
  )
}

function SourcePreviewSection({
  preview,
  onRetry,
  onOpenManga
}: {
  preview: SourcePreview
  onRetry: () => void
  onOpenManga: (source: InstalledSource, manga: SourceManga) => void
}) {
  return (
    <section className="min-w-0 space-y-4">
      <div className="flex items-end justify-between gap-3">
        <div className="min-w-0 space-y-1.5">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <h2 className="truncate text-xl font-semibold">{preview.source.info.name}</h2>
            <Badge variant="outline">{preview.listing.name}</Badge>
          </div>
        </div>

        <Button asChild variant="outline" size="sm">
          <Link
            to="/explore/list"
            search={{
              sourceId: preview.source.info.id,
              listingId: preview.listing.id,
              page: 1
            }}
          >
            更多
            <ArrowRightIcon className="size-4" />
          </Link>
        </Button>
      </div>

      {preview.error ? (
        <div className="flex items-center justify-between gap-3 rounded-xl border border-destructive/30 bg-destructive/5 px-4 py-3">
          <p className="line-clamp-2 text-sm text-destructive">{preview.error}</p>
          <Button type="button" variant="outline" size="sm" onClick={onRetry}>
            重试
          </Button>
        </div>
      ) : preview.entries.length === 0 ? (
        <p className="rounded-xl border border-dashed px-4 py-6 text-center text-sm text-muted-foreground">
          这个源暂时没有探索内容
        </p>
      ) : (
        <ComicRail>
          {preview.entries.slice(0, 8).map(manga => {
            const comic = mapSourceManga(manga)
            return (
              <ComicRailItem key={`${preview.source.info.id}:${manga.key}`}>
                <ComicCard
                  comic={comic}
                  ratio="square"
                  onOpen={() => onOpenManga(preview.source, manga)}
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

async function loadSourcePreviews(sources: InstalledSource[]): Promise<SourcePreview[]> {
  return Promise.all(
    sources.map(async source => {
      const listing = source.listings[0]
      if (!listing) {
        throw new Error(`漫画源 ${source.info.id} 没有可用列表`)
      }
      try {
        const result = await getSourceListing(source.info.id, listing)
        return {
          source,
          listing,
          entries: result.entries,
          error: null
        }
      } catch (error) {
        return {
          source,
          listing,
          entries: [],
          error: error instanceof Error ? error.message : String(error)
        }
      }
    })
  )
}
