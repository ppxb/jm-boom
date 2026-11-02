import { useNavigate, useRouter } from '@tanstack/react-router'
import { useCallback, useMemo, useRef } from 'react'

import { ReaderBottomBar, ReaderTopBar } from './reader-bars'
import { ReaderHotZones } from './reader-hot-zones'
import { ReaderImageWindow } from './reader-image'
import { ReaderError, ReaderLoading } from './reader-state'
import { ReaderStripWindow } from './reader-strip-window'
import type { ReaderSearch } from './types'
import { useReaderAutoRead } from './use-reader-auto-read'
import { useReaderChapterInfo } from './use-reader-chapter-info'
import { useReaderHistorySync } from './use-reader-history-sync'
import { useReaderKeyboardNavigation } from './use-reader-keyboard-navigation'
import { useNextChapterPrefetch } from './use-next-chapter-prefetch'
import { useReaderPreload } from './use-reader-preload'
import { useReaderSession } from './use-reader-session'
import { useReaderToolbarVisibility } from './use-reader-toolbar-visibility'
import { READER } from '@/lib/constants'
import { cn } from 'cn'
import { useSettingsStore } from '@/stores/settings-store'

export function ReaderPage({ comicId, search }: { comicId: string; search: ReaderSearch }) {
  const navigate = useNavigate()
  const router = useRouter()
  const readerReadMode = useSettingsStore(state => state.readerReadMode)
  const readerPageDirection = useSettingsStore(state => state.readerPageDirection)
  const readerDoublePageMode = useSettingsStore(state => state.readerDoublePageMode)
  const readerCompactMenuEnabled = useSettingsStore(state => state.readerCompactMenuEnabled)
  const showTransitions = useSettingsStore(state => state.readerChapterTransitions)
  const isStripMode = readerReadMode === 'strip'
  const isDoublePageMode = !isStripMode && readerDoublePageMode
  const stripScrollRef = useRef<HTMLDivElement | null>(null)
  const {
    isVisible: isToolbarVisible,
    toggle: toggleToolbar,
    hide: hideToolbar
  } = useReaderToolbarVisibility(false)
  const {
    albumId,
    title,
    author,
    coverUrl,
    chapters,
    chapter: fallbackChapter
  } = useReaderChapterInfo({ comicId, search })
  const readingChapters = useMemo(() => [...chapters].reverse(), [chapters])
  const syncLocation = useCallback(
    (readId: string, index: number) =>
      navigate({
        to: '/reader/$comicId',
        params: { comicId: readId },
        search: { albumId, page: index + 1 },
        replace: true,
        resetScroll: false
      }),
    [albumId, navigate]
  )
  const session = useReaderSession({
    comicId,
    initialIndex: search.page ? search.page - 1 : 0,
    pageStep: isDoublePageMode ? 2 : 1,
    chapters: readingChapters,
    onLocationChange: syncLocation
  })
  const {
    activeReadId,
    currentIndex,
    globalIndex,
    pageCount,
    pages,
    pageSrc,
    pageWindow,
    visiblePageCount,
    navigationCommand,
    isManifestLoading,
    manifestError,
    isFetching,
    previousChapter,
    nextChapter,
    goToPreviousPage,
    goToNextPage,
    goToPage,
    goToChapter,
    observePage,
    observeVisibleRange,
    onPageLoaded,
    onPageReady,
    loadBoundary,
    retry
  } = session
  const chapterTitles = useMemo(
    () => new Map(chapters.map(item => [item.id, item.title])),
    [chapters]
  )
  const chapter = chapterTitles.get(activeReadId) || fallbackChapter
  useReaderPreload({ readId: albumId, pages, currentIndex: globalIndex })
  useNextChapterPrefetch({ currentIndex, pageCount, nextChapter })
  useReaderHistorySync({
    comicId: activeReadId,
    albumId,
    title,
    author,
    coverUrl,
    chapter,
    currentIndex,
    pageCount
  })

  const goBack = useCallback(() => {
    if (window.history.length > 1) {
      router.history.back()
    } else if (albumId) {
      void navigate({ to: '/comic/$comicId', params: { comicId: albumId }, replace: true })
    } else {
      void navigate({ to: '/' })
    }
  }, [albumId, navigate, router])
  const previousFromContent = useCallback(() => {
    hideToolbar()
    goToPreviousPage()
  }, [goToPreviousPage, hideToolbar])
  const nextFromContent = useCallback(() => {
    hideToolbar()
    goToNextPage()
  }, [goToNextPage, hideToolbar])
  const openChapter = useCallback(
    (readId: string, index = 0) => {
      hideToolbar()
      void goToChapter(readId, index)
    },
    [goToChapter, hideToolbar]
  )
  const loadNextBoundary = useCallback(() => {
    if (!session.nextBoundary.loading && !session.nextBoundary.error) loadBoundary(1)
  }, [loadBoundary, session.nextBoundary.error, session.nextBoundary.loading])
  const scrollStripBy = useCallback(
    (direction: -1 | 1) => {
      const container = stripScrollRef.current
      if (!container) return
      const atEdge =
        direction > 0
          ? container.scrollHeight - container.clientHeight - container.scrollTop <=
            READER.STRIP_SCROLL_THRESHOLD
          : container.scrollTop <= READER.STRIP_SCROLL_THRESHOLD
      if (atEdge) loadBoundary(direction)
      container.scrollBy({
        top: direction * Math.max(220, container.clientHeight * 0.35),
        behavior: 'smooth'
      })
    },
    [loadBoundary]
  )

  useReaderKeyboardNavigation({
    readMode: readerReadMode,
    pageDirection: readerPageDirection,
    onPrevious: goToPreviousPage,
    onNext: goToNextPage,
    onScrollPrevious: () => scrollStripBy(-1),
    onScrollNext: () => scrollStripBy(1),
    onBack: goBack,
    onNavigate: hideToolbar
  })
  useReaderAutoRead({
    readMode: readerReadMode,
    pageCount,
    controlsVisible: isToolbarVisible,
    canAdvance: session.isPageReady && !session.jump && !manifestError,
    stripScrollRef,
    onNextPage: goToNextPage,
    onStripEnd: loadNextBoundary
  })

  return (
    <main
      className="relative flex h-dvh overflow-hidden bg-neutral-950 text-neutral-50"
      onClick={toggleToolbar}
      onTouchMove={hideToolbar}
    >
      <ReaderTopBar
        fallbackReadId={activeReadId}
        title={title}
        chapter={chapter}
        isFetching={isFetching}
        visible={isToolbarVisible}
        onBack={goBack}
        onRetry={retry}
      />
      {isStripMode ? null : (
        <ReaderHotZones
          pageDirection={readerPageDirection}
          onPrevious={previousFromContent}
          onNext={nextFromContent}
        />
      )}
      <section
        className={cn(
          'flex min-w-0 flex-1 items-center justify-center',
          isStripMode ? 'h-dvh' : null
        )}
      >
        {isManifestLoading ? (
          <ReaderLoading label="正在加载阅读信息" />
        ) : manifestError ? (
          <ReaderError title="阅读信息加载失败" description={manifestError.message} />
        ) : pageCount <= 0 ? (
          <ReaderError title="暂无图片" description="当前章节没有可展示的图片" />
        ) : isStripMode ? (
          <ReaderStripWindow
            containerRef={stripScrollRef}
            pages={pages}
            currentIndex={globalIndex}
            navigationCommand={navigationCommand}
            onCurrentIndexChange={observePage}
            onVisibleRangeChange={observeVisibleRange}
            onUserScroll={hideToolbar}
            previousBoundary={session.previousBoundary}
            nextBoundary={session.nextBoundary}
            onLoadBoundary={loadBoundary}
            onPageLoaded={onPageLoaded}
            onPageReady={onPageReady}
            chapterTitles={chapterTitles}
            showTransitions={showTransitions}
          />
        ) : pageSrc ? (
          <ReaderImageWindow
            pages={pageWindow}
            currentIndex={globalIndex}
            pageCount={pages.length}
            doublePageMode={isDoublePageMode}
            visiblePageCount={visiblePageCount}
            pageDirection={readerPageDirection}
            onPageLoaded={onPageLoaded}
            onPageReady={onPageReady}
          />
        ) : null}
      </section>
      {!isStripMode && showTransitions && currentIndex === 0 && chapter ? (
        <div className="pointer-events-none absolute inset-x-0 top-20 z-20 text-center text-xs text-neutral-400">
          {chapter}
        </div>
      ) : null}
      {session.jump ? (
        <div
          role={session.jump.error ? 'alert' : 'status'}
          className="absolute bottom-28 left-1/2 z-30 flex max-w-[90vw] -translate-x-1/2 items-center gap-3 rounded-lg bg-neutral-900/95 px-4 py-3 text-sm"
          onClick={event => event.stopPropagation()}
        >
          <span>{session.jump.loading ? '正在加载章节…' : session.jump.error}</span>
          {session.jump.error ? (
            <button type="button" className="shrink-0 underline" onClick={retry}>
              重试
            </button>
          ) : null}
        </div>
      ) : null}
      <ReaderBottomBar
        title={title}
        currentReadId={activeReadId}
        previousChapter={previousChapter}
        nextChapter={nextChapter}
        chapters={chapters}
        albumId={albumId}
        currentIndex={currentIndex}
        pageCount={pageCount}
        doublePageMode={visiblePageCount === 2}
        compactMenuEnabled={readerCompactMenuEnabled}
        visible={isToolbarVisible && pageCount > 0}
        onPageChange={goToPage}
        onChapterChange={openChapter}
      />
    </main>
  )
}
