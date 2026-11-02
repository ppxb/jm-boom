import { useQuery } from '@tanstack/react-query'
import { useMemo, useState } from 'react'

import { BackTopButton } from '@/components/back-top-button'
import { EmptyState } from '@/components/empty-state'
import { PageBackButton } from '@/components/page-back-button'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import type { ComicDetail } from '@/domain/comic'
import { getComicDetail, getComicState, type ComicStateResult } from '@/lib/api/comic'
import { sortComicChapters } from '@/lib/comic'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { ChaptersSection } from './chapters'
import { CommentsDrawer } from './comments'
import { ComicHero } from './hero'
import { RelatedPanel } from './related'
import { ComicDetailSkeleton } from './shared'
import { ComicDownloadDrawer } from './download-drawer'
import { resolveComicReadingTarget } from './reading-target'
import { useComicDownload } from './use-comic-download'
import { useComicFavorite } from './use-comic-favorite'
import { useComicReaderPreload } from './use-comic-reader-preload'

export function ComicDetailPage({ comicId }: { comicId: string }) {
  const detail = useQuery({
    queryKey: queryKeys.comicDetail(comicId),
    queryFn: () => getComicDetail(comicId),
    staleTime: CACHE.DETAIL_STALE_TIME,
    gcTime: CACHE.DETAIL_GC_TIME
  })
  const comicState = useQuery({
    queryKey: queryKeys.comicState(comicId),
    queryFn: () => getComicState(comicId),
    staleTime: 10_000,
    refetchOnMount: true,
    refetchOnWindowFocus: true
  })

  return (
    <main className="min-h-screen bg-background px-4 pt-6 pb-24 text-foreground sm:px-6 md:pb-6 lg:px-8">
      <div className="mx-auto max-w-7xl space-y-8">
        <PageBackButton />

        {detail.isLoading ? (
          <ComicDetailSkeleton />
        ) : detail.isError ? (
          <EmptyState
            emoji="Ò︵Ó"
            title="数据加载失败"
            actions={
              <Button type="button" variant="outline" size="sm" onClick={() => detail.refetch()}>
                重试
              </Button>
            }
          />
        ) : detail.data == null ? (
          <EmptyState emoji="(･o･;)" title="暂无详情" />
        ) : (
          <>
            {comicState.isError ? (
              <Card size="sm" role="alert">
                <CardContent className="flex flex-wrap items-center justify-between gap-3">
                  <p className="text-muted-foreground">
                    收藏与阅读进度加载失败。
                    {comicState.data
                      ? '当前显示上次记录，可重试更新。'
                      : '请重试，或从章节列表开始阅读。'}
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={comicState.isFetching}
                    onClick={() => void comicState.refetch()}
                  >
                    {comicState.isFetching ? '重试中…' : '重试状态'}
                  </Button>
                </CardContent>
              </Card>
            ) : !comicState.data && comicState.isLoading ? (
              <p role="status" className="text-sm text-muted-foreground">
                正在读取收藏与阅读进度…
              </p>
            ) : null}
            <ComicDetailView comic={detail.data.comic} state={comicState.data} />
          </>
        )}
      </div>
      <BackTopButton />
    </main>
  )
}

function ComicDetailView({
  comic,
  state
}: {
  comic: ComicDetail
  state: ComicStateResult | undefined
}) {
  const readingHistory = state?.history ?? undefined
  const [isCommentsOpen, setIsCommentsOpen] = useState(false)
  const [chaptersDescending, setChaptersDescending] = useState(true)
  const albumId = comic.id
  const readingChapters = useMemo(() => sortComicChapters(comic.chapters), [comic.chapters])
  const sortedChapters = useMemo(
    () => (chaptersDescending ? readingChapters : [...readingChapters].reverse()),
    [chaptersDescending, readingChapters]
  )
  const readingTarget = useMemo(
    () => (state ? resolveComicReadingTarget(comic, readingChapters, readingHistory) : null),
    [comic, readingChapters, readingHistory, state]
  )
  const favorite = useComicFavorite({ comic, state })
  const download = useComicDownload(comic, sortedChapters)
  const handleCoverSettled = useComicReaderPreload(comic, readingTarget)

  return (
    <div className="space-y-10">
      <ComicHero
        comic={comic}
        readingTarget={readingTarget}
        isFavorite={favorite.isFavorite}
        onCommentsClick={() => setIsCommentsOpen(true)}
        onDownloadClick={download.start}
        onFavoriteClick={favorite.toggle}
        onCoverSettled={handleCoverSettled}
        downloadBusy={download.isPending}
        favoriteBusy={favorite.isPending}
      />

      <ChaptersSection
        albumId={albumId}
        comicId={comic.id}
        totalPhotos={comic.totalPhotos}
        sortedChapters={sortedChapters}
        descending={chaptersDescending}
        onToggleSort={() => setChaptersDescending(current => !current)}
      />

      <RelatedPanel items={comic.relatedComics} />

      <CommentsDrawer
        comicId={comic.id}
        total={comic.commentCount}
        open={isCommentsOpen}
        onOpenChange={setIsCommentsOpen}
      />
      <ComicDownloadDrawer
        open={download.isOpen}
        onOpenChange={download.setIsOpen}
        comicTitle={comic.title}
        chapters={download.chapters}
        isSubmitting={download.isPending}
        onConfirm={download.submit}
      />
    </div>
  )
}
