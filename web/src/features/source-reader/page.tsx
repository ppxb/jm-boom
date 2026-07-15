import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useNavigate, useRouter } from '@tanstack/react-router'
import { useCallback, useRef } from 'react'

import { cn } from '@/lib/utils'
import {
  createSourceMangaStub,
  getSourceManga,
  getSourceReaderPages,
  type SourceChapter,
  type SourceManga
} from '@/lib/api/source'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { useSettingsStore } from '@/stores/settings-store'
import { ReaderTopBar } from '@/features/reader/reader-bars'
import { ReaderHotZones } from '@/features/reader/reader-hot-zones'
import { ReaderImageWindow } from '@/features/reader/reader-image'
import { ReaderProgressSlider } from '@/features/reader/reader-progress-slider'
import { ReaderSettingsMenu } from '@/features/reader/reader-settings-menu'
import { ReaderError, ReaderLoading } from '@/features/reader/reader-state'
import { ReaderStripWindow } from '@/features/reader/reader-strip-window'
import { useReaderAutoRead } from '@/features/reader/use-reader-auto-read'
import { useReaderKeyboardNavigation } from '@/features/reader/use-reader-keyboard-navigation'
import { useReaderPageSession } from '@/features/reader/use-reader-session'
import { useReaderPreload } from '@/features/reader/use-reader-preload'
import { useReaderToolbarVisibility } from '@/features/reader/use-reader-toolbar-visibility'

const EMPTY_PAGES: Awaited<ReturnType<typeof getSourceReaderPages>> = []

export function SourceReaderPage({
  sourceId,
  mangaKey,
  chapterKey,
  initialPage
}: {
  sourceId: string
  mangaKey: string
  chapterKey: string
  initialPage?: number
}) {
  const navigate = useNavigate()
  const router = useRouter()
  const queryClient = useQueryClient()
  const mangaQueryKey = queryKeys.sourceManga(sourceId, mangaKey)
  const mangaSeed =
    queryClient.getQueryData<SourceManga>(mangaQueryKey) ?? createSourceMangaStub(mangaKey)
  const mangaQuery = useQuery({
    queryKey: mangaQueryKey,
    queryFn: () => getSourceManga(sourceId, mangaSeed),
    initialData: mangaSeed,
    initialDataUpdatedAt: mangaSeed.chapters == null ? 0 : undefined,
    staleTime: CACHE.DETAIL_STALE_TIME,
    gcTime: CACHE.DETAIL_GC_TIME,
    refetchOnWindowFocus: false
  })
  const manga = mangaQuery.data
  const chapter = manga.chapters?.find(item => item.key === chapterKey) ?? null
  const pagesQuery = useQuery({
    queryKey: queryKeys.sourcePages(sourceId, mangaKey, chapterKey),
    queryFn: () => getSourceReaderPages(sourceId, manga, chapter as SourceChapter),
    enabled: chapter != null,
    staleTime: CACHE.READER_STALE_TIME,
    gcTime: CACHE.READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const pages = pagesQuery.data ?? EMPTY_PAGES
  const readerReadMode = useSettingsStore(state => state.readerReadMode)
  const readerPageDirection = useSettingsStore(state => state.readerPageDirection)
  const readerDoublePageMode = useSettingsStore(state => state.readerDoublePageMode)
  const isStripMode = readerReadMode === 'strip'
  const isDoublePageMode = !isStripMode && readerDoublePageMode
  const pageStep = isDoublePageMode ? 2 : 1
  const stripScrollRef = useRef<HTMLDivElement | null>(null)
  const { isVisible, toggle, hide } = useReaderToolbarVisibility(false)
  const session = useReaderPageSession({
    scopeId: `${sourceId}:${mangaKey}:${chapterKey}`,
    pages,
    initialIndex: initialPage ? initialPage - 1 : 0,
    pageStep
  })
  const isLoading = mangaQuery.isFetching && chapter == null ? true : pagesQuery.isLoading
  const error = mangaQuery.isError
    ? mangaQuery.error
    : chapter == null && !mangaQuery.isFetching
      ? new Error('漫画源没有返回当前章节，请返回详情页重新选择')
      : pagesQuery.isError
        ? pagesQuery.error
        : null

  useReaderPreload({
    readId: `${sourceId}:${chapterKey}`,
    pages,
    currentIndex: session.currentIndex
  })

  const goBack = useCallback(() => {
    if (window.history.length > 1) {
      router.history.back()
      return
    }
    void navigate({
      to: '/comic/$comicId',
      params: { comicId: mangaKey },
      search: { sourceId },
      replace: true
    })
  }, [mangaKey, navigate, router, sourceId])
  const scrollStripBy = useCallback(
    (direction: 1 | -1) => {
      const container = stripScrollRef.current
      if (!container) return
      container.scrollBy({
        top: direction * Math.max(220, container.clientHeight * 0.35),
        behavior: 'smooth'
      })
    },
    []
  )

  useReaderKeyboardNavigation({
    readMode: readerReadMode,
    pageDirection: readerPageDirection,
    onPrevious: session.goToPreviousPage,
    onNext: session.goToNextPage,
    onScrollPrevious: () => scrollStripBy(-1),
    onScrollNext: () => scrollStripBy(1),
    onBack: goBack,
    onNavigate: hide
  })
  useReaderAutoRead({
    readMode: readerReadMode,
    pageStep,
    pageCount: session.pageCount,
    currentIndex: session.currentIndex,
    controlsVisible: isVisible,
    canAdvance: !isLoading && error == null && (isStripMode || session.pageSrc.length > 0),
    stripScrollRef,
    onNextPage: session.goToNextPage
  })

  function retry() {
    void mangaQuery.refetch()
    if (chapter) void pagesQuery.refetch()
  }

  return (
    <main
      className="relative flex h-screen overflow-hidden bg-neutral-950 text-neutral-50"
      onClick={toggle}
      onTouchMove={hide}
    >
      <ReaderTopBar
        fallbackReadId={chapterKey}
        title={manga.title}
        chapter={chapter ? formatChapterTitle(chapter) : chapterKey}
        isFetching={mangaQuery.isFetching || pagesQuery.isFetching}
        visible={isVisible}
        onBack={goBack}
        onRetry={retry}
      />

      {isStripMode ? null : (
        <ReaderHotZones
          pageDirection={readerPageDirection}
          onPrevious={() => {
            hide()
            session.goToPreviousPage()
          }}
          onNext={() => {
            hide()
            session.goToNextPage()
          }}
        />
      )}

      <section
        className={cn(
          'flex min-w-0 flex-1 items-center justify-center',
          isStripMode ? 'h-screen' : null
        )}
      >
        {isLoading ? (
          <ReaderLoading label="正在从漫画源加载页面" />
        ) : error ? (
          <ReaderError title="漫画源阅读加载失败" description={error.message} />
        ) : session.pageCount <= 0 ? (
          <ReaderError title="暂无图片" description="当前章节没有可展示的图片" />
        ) : isStripMode ? (
          <ReaderStripWindow
            key={`${sourceId}:${chapterKey}`}
            containerRef={stripScrollRef}
            pages={pages}
            currentIndex={session.currentIndex}
            navigationCommand={session.navigationCommand}
            onCurrentIndexChange={session.observePage}
            onUserScroll={hide}
          />
        ) : session.pageSrc.length > 0 ? (
          <ReaderImageWindow
            pages={session.pageWindow}
            currentIndex={session.currentIndex}
            pageCount={session.pageCount}
            doublePageMode={isDoublePageMode}
            pageDirection={readerPageDirection}
          />
        ) : (
          <ReaderError title="暂无图片" description="当前页没有可展示的图片" />
        )}
      </section>

      <SourceReaderBottomBar
        currentIndex={session.currentIndex}
        pageCount={session.pageCount}
        doublePageMode={isDoublePageMode}
        visible={isVisible}
        onPageChange={session.goToPage}
      />
    </main>
  )
}

function SourceReaderBottomBar({
  currentIndex,
  pageCount,
  doublePageMode,
  visible,
  onPageChange
}: {
  currentIndex: number
  pageCount: number
  doublePageMode: boolean
  visible: boolean
  onPageChange: (index: number) => void
}) {
  if (pageCount <= 0) return null
  const pageLabel =
    doublePageMode && currentIndex + 1 < pageCount
      ? `${currentIndex + 1}-${currentIndex + 2} / ${pageCount}`
      : `${currentIndex + 1} / ${pageCount}`

  if (!visible) {
    return (
      <div className="pointer-events-none absolute bottom-24 left-1/2 z-30 flex h-8 w-24 -translate-x-1/2 items-center justify-center rounded-2xl border border-border/60 bg-background/85 px-3 text-xs text-muted-foreground backdrop-blur sm:bottom-8">
        <span className="tabular-nums">{pageLabel}</span>
      </div>
    )
  }

  return (
    <footer
      className="absolute bottom-24 left-1/2 z-30 flex w-[360px] max-w-[calc(100vw-24px)] -translate-x-1/2 flex-col gap-3 rounded-2xl border border-border/60 bg-background/85 p-4 text-foreground backdrop-blur sm:bottom-8 sm:w-[480px] sm:max-w-[calc(100vw-48px)] sm:p-3"
      onClick={event => event.stopPropagation()}
      onTouchMove={event => event.stopPropagation()}
    >
      <ReaderProgressSlider
        currentIndex={currentIndex}
        pageCount={pageCount}
        onPageChange={onPageChange}
      />
      <div className="flex items-center justify-between gap-3">
        <ReaderSettingsMenu />
        <span className="text-sm text-muted-foreground tabular-nums sm:text-xs">{pageLabel}</span>
      </div>
    </footer>
  )
}

function formatChapterTitle(chapter: SourceChapter) {
  if (chapter.title) return chapter.title
  if (chapter.chapterNumber != null) return `第 ${chapter.chapterNumber} 章`
  return chapter.key
}
