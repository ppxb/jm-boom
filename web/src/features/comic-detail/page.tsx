import { useInfiniteQuery, useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { useEffect, useMemo, useState } from 'react'

import { BackTopButton } from '@/components/back-top-button'
import { PageBackButton } from '@/components/page-back-button'
import type { ComicDetail } from '@/domain/comic'
import { getComicComments, getComicDetail } from '@/lib/api/comic'
import { SINGLE_CHAPTER_TITLE, sortComicChapters } from '@/lib/comic'
import { getComicReadManifest } from '@/lib/api/reader'
import { CACHE, READER } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { clearReaderPreloadScope, setReaderPreloadScope } from '@/lib/reader-preload'
import { ChaptersSection } from './chapters'
import { CommentsDrawer } from './comments'
import { ComicHero } from './hero'
import { RelatedPanel } from './related'
import { ComicDetailSkeleton } from './shared'
import { EmptyState } from '@/components/empty-state'
import { Button } from '@/components/ui/button'
import {
  ComicDownloadDrawer,
  toDownloadChapterOptions,
  type DownloadChapterOption
} from './download-drawer'
import { enqueueComicDownload } from '@/lib/api/download'
import { addFavorite, listFavorites, removeFavorite } from '@/lib/api/favorite'
import { useReadingHistoryStore } from '@/stores/reading-history-store'
import { useSettingsStore } from '@/stores/settings-store'
import { resolveComicReadingTarget } from './reading-target'

export function ComicDetailPage({ comicId }: { comicId: string }) {
  const detail = useQuery({
    queryKey: queryKeys.comicDetail(comicId),
    queryFn: () => getComicDetail(comicId),
    staleTime: CACHE.DETAIL_STALE_TIME,
    gcTime: CACHE.DETAIL_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
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
          <ComicDetailView comic={detail.data.comic} />
        )}
      </div>
      <BackTopButton />
    </main>
  )
}
function ComicDetailView({ comic }: { comic: ComicDetail }) {
  const queryClient = useQueryClient()
  const favorites = useQuery({
    queryKey: queryKeys.favorites(),
    queryFn: listFavorites,
    staleTime: 0,
    refetchOnMount: 'always',
    refetchOnWindowFocus: true
  })
  const isFavorite = favorites.data?.items.some(item => item.id === comic.id) ?? false
  const readingHistory = useReadingHistoryStore(state =>
    state.items.find(item => item.id === comic.id)
  )
  const hideCovers = useSettingsStore(state => state.hideCovers)
  const [isCommentsOpen, setIsCommentsOpen] = useState(false)
  const [isDownloadOpen, setIsDownloadOpen] = useState(false)
  const [settledCoverUrl, setSettledCoverUrl] = useState('')
  const albumId = comic.id
  const readingTarget = useMemo(
    () => resolveComicReadingTarget(comic, readingHistory),
    [comic, readingHistory]
  )
  const readerPreloadScope = `detail:${comic.id}`
  const isCoverSettled = hideCovers || comic.image.length === 0 || settledCoverUrl === comic.image
  const downloadChapters = useMemo(() => {
    const chapters = sortComicChapters(comic.chapters)

    if (chapters.length === 0) {
      return [
        {
          chapterId: comic.id,
          title: SINGLE_CHAPTER_TITLE,
          order: 1
        }
      ]
    }

    return toDownloadChapterOptions(chapters)
  }, [comic.chapters, comic.id])

  useEffect(() => {
    if (!isCoverSettled) {
      return
    }

    const readId = readingTarget.readId.trim()

    if (readId.length === 0) {
      return
    }

    let isActive = true

    void queryClient
      .fetchQuery({
        queryKey: queryKeys.readerManifest(readId),
        queryFn: () => getComicReadManifest({ readId }),
        staleTime: CACHE.READER_STALE_TIME,
        gcTime: CACHE.READER_GC_TIME
      })
      .then(manifest => {
        if (!isActive) {
          return
        }

        const initialPageIndex = Math.max((readingTarget.page ?? 1) - 1, 0)
        const startIndex = Math.min(initialPageIndex, Math.max(manifest.pages.length - 1, 0))
        const paths = manifest.pages
          .slice(startIndex, startIndex + READER.PREFETCH_AHEAD_PAGES)
          .map(page => page.path)
        setReaderPreloadScope(readerPreloadScope, paths)
      })
      .catch(error => {
        if (isActive && import.meta.env.DEV) {
          console.debug('Comic detail reader manifest prefetch failed', error)
        }
      })

    return () => {
      isActive = false
      clearReaderPreloadScope(readerPreloadScope)
    }
  }, [isCoverSettled, queryClient, readerPreloadScope, readingTarget.page, readingTarget.readId])

  const favoriteMutation = useMutation({
    mutationFn: () =>
      isFavorite
        ? removeFavorite(comic.id)
        : addFavorite({
            id: comic.id,
            title: comic.title,
            author: comic.authors.join(' / '),
            description: comic.description,
            image: comic.image,
            tags: comic.tags
          }),
    onSuccess: result => {
      queryClient.setQueryData(queryKeys.favorites(), result)
      toast.success(isFavorite ? '已取消收藏' : '已添加收藏')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '收藏操作失败')
    }
  })
  const downloadMutation = useMutation({
    mutationFn: (chapters: DownloadChapterOption[]) =>
      enqueueComicDownload({
        albumId,
        comicTitle: comic.title,
        chapters
      }),
    onSuccess: result => {
      queryClient.setQueryData(queryKeys.downloadTasks(), result)
      setIsDownloadOpen(false)
      toast.success('已加入下载队列，可在下载页查看进度')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '下载任务创建失败')
    }
  })
  const commentsQuery = useInfiniteQuery({
    queryKey: queryKeys.comicComments(comic.id),
    queryFn: ({ pageParam }) => getComicComments({ comicId: comic.id, page: pageParam }),
    initialPageParam: 1,
    enabled: isCommentsOpen,
    staleTime: CACHE.COMMENTS_STALE_TIME,
    gcTime: CACHE.COMMENTS_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    getNextPageParam: (lastPage, allPages) => {
      const loadedCount = allPages.reduce((sum, page) => sum + page.comments.length, 0)

      if (lastPage.comments.length === 0 || loadedCount >= lastPage.total) {
        return undefined
      }

      return lastPage.page + 1
    }
  })
  const comments = useMemo(
    () => commentsQuery.data?.pages.flatMap(page => page.comments) ?? [],
    [commentsQuery.data]
  )
  const commentTotal = commentsQuery.data?.pages[0]?.total ?? comic.commentCount

  function handleDownloadClick() {
    if (downloadChapters.length <= 1) {
      downloadMutation.mutate(downloadChapters)
      return
    }

    setIsDownloadOpen(true)
  }

  return (
    <div className="space-y-10">
      <ComicHero
        comic={comic}
        readingTarget={readingTarget}
        isFavorite={isFavorite}
        onCommentsClick={() => setIsCommentsOpen(true)}
        onDownloadClick={handleDownloadClick}
        onFavoriteClick={() => favoriteMutation.mutate()}
        onCoverSettled={() => setSettledCoverUrl(comic.image)}
        downloadBusy={downloadMutation.isPending}
        favoriteBusy={favorites.isLoading || favoriteMutation.isPending}
      />

      <ChaptersSection albumId={albumId} comicId={comic.id} chapters={comic.chapters} />

      <RelatedPanel items={comic.relatedComics} />

      <CommentsDrawer
        open={isCommentsOpen}
        onOpenChange={setIsCommentsOpen}
        state={{
          isLoading: commentsQuery.isLoading,
          isFetchingNextPage: commentsQuery.isFetchingNextPage,
          isError: commentsQuery.isError,
          errorMessage: commentsQuery.error?.message,
          total: commentTotal,
          comments,
          hasNextPage: commentsQuery.hasNextPage,
          onRetry: () => commentsQuery.refetch(),
          onLoadMore: () => commentsQuery.fetchNextPage({ cancelRefetch: false })
        }}
      />
      <ComicDownloadDrawer
        open={isDownloadOpen}
        onOpenChange={setIsDownloadOpen}
        comicTitle={comic.title}
        chapters={downloadChapters}
        isSubmitting={downloadMutation.isPending}
        onConfirm={chapters => downloadMutation.mutate(chapters)}
      />
    </div>
  )
}
