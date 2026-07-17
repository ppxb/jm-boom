import { useNavigate, useRouter } from '@tanstack/react-router'
import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react'

import { ReaderBottomBar, ReaderTopBar } from './reader-bars'
import { ReaderHotZones } from './reader-hot-zones'
import { ReaderImageWindow } from './reader-image'
import { ReaderError, ReaderLoading } from './reader-state'
import { ReaderStripWindow } from './reader-strip-window'
import { toReaderChapterSearch } from './reader-chapter-link'
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
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings-store'

export function ReaderPage({ comicId, search }: { comicId: string; search: ReaderSearch }) {
  const navigate = useNavigate()
  const router = useRouter()
  const readerReadMode = useSettingsStore(state => state.readerReadMode)
  const readerPageDirection = useSettingsStore(state => state.readerPageDirection)
  const readerDoublePageMode = useSettingsStore(state => state.readerDoublePageMode)
  const isStripMode = readerReadMode === 'strip'
  const isDoublePageMode = !isStripMode && readerDoublePageMode
  const pageStep = isDoublePageMode ? 2 : 1
  const stripScrollRef = useRef<HTMLDivElement | null>(null)
  const nextChapterNavigationRef = useRef<string | null>(null)
  const [stripPrefetchRequested, setStripPrefetchRequested] = useState(false)
  const {
    isVisible: isToolbarVisible,
    toggle: toggleToolbar,
    hide: hideToolbar
  } = useReaderToolbarVisibility(false)
  const initialPageIndex = search.page ? search.page - 1 : 0
  const { albumId, title, author, coverUrl, chapter, chapters, previousChapter, nextChapter } =
    useReaderChapterInfo({
      comicId,
      search
    })
  const {
    currentIndex,
    pageCount,
    pages,
    pageSrc,
    pageWindow,
    navigationCommand,
    isManifestLoading,
    manifestError,
    isFetching,
    isLastPage,
    goToPreviousPage,
    goToNextPage,
    goToPage,
    observePage,
    retry
  } = useReaderSession({ comicId, initialIndex: initialPageIndex, pageStep })
  const availableNextChapter = nextChapter

  useReaderPreload({ readId: comicId, pages, currentIndex })

  useReaderHistorySync({
    comicId,
    albumId,
    title,
    author,
    coverUrl,
    chapter,
    currentIndex,
    pageCount
  })

  useLayoutEffect(() => {
    hideToolbar()
  }, [comicId, hideToolbar])

  useEffect(() => {
    nextChapterNavigationRef.current = null
  }, [comicId])

  const goBack = useCallback(() => {
    if (window.history.length > 1) {
      router.history.back()
      return
    }

    if (albumId.length > 0) {
      void navigate({ to: '/comic/$comicId', params: { comicId: albumId }, replace: true })
      return
    }

    void navigate({ to: '/' })
  }, [albumId, navigate, router])
  const goToNextChapter = useCallback(() => {
    if (!availableNextChapter) {
      return false
    }

    if (nextChapterNavigationRef.current === availableNextChapter.id) {
      return true
    }

    nextChapterNavigationRef.current = availableNextChapter.id
    void navigate({
      to: '/reader/$comicId',
      params: { comicId: availableNextChapter.id },
      replace: true,
      resetScroll: true,
      search: toReaderChapterSearch({
        albumId
      })
    }).catch(() => {
      nextChapterNavigationRef.current = null
    })

    return true
  }, [albumId, availableNextChapter, navigate])
  const goToNextPageOrChapter = useCallback(() => {
    if (isLastPage && goToNextChapter()) {
      return
    }

    goToNextPage()
  }, [goToNextChapter, goToNextPage, isLastPage])
  const goToPreviousPageFromContent = useCallback(() => {
    hideToolbar()
    goToPreviousPage()
  }, [goToPreviousPage, hideToolbar])
  const goToNextPageOrChapterFromContent = useCallback(() => {
    hideToolbar()
    goToNextPageOrChapter()
  }, [goToNextPageOrChapter, hideToolbar])
  const requestStripPrefetch = useCallback(() => {
    setStripPrefetchRequested(true)
  }, [])
  const scrollStripBy = useCallback(
    (direction: 1 | -1) => {
      const container = stripScrollRef.current

      if (!container) {
        return
      }

      const maxScrollTop = Math.max(container.scrollHeight - container.clientHeight, 0)
      const isAtEnd = maxScrollTop - container.scrollTop <= READER.STRIP_SCROLL_THRESHOLD

      if (direction > 0 && isAtEnd && goToNextChapter()) {
        return
      }

      container.scrollBy({
        top: direction * Math.max(220, container.clientHeight * 0.35),
        behavior: 'smooth'
      })
    },
    [goToNextChapter]
  )

  useReaderKeyboardNavigation({
    readMode: readerReadMode,
    pageDirection: readerPageDirection,
    onPrevious: goToPreviousPage,
    onNext: goToNextPageOrChapter,
    onScrollPrevious: () => scrollStripBy(-1),
    onScrollNext: () => scrollStripBy(1),
    onBack: goBack,
    onNavigate: hideToolbar
  })
  useNextChapterPrefetch({
    currentIndex,
    pageCount,
    nextChapter: availableNextChapter,
    stripPrefetchRequested
  })

  const showReaderTopBar = isToolbarVisible
  const showReaderBottomBar = isToolbarVisible && pageCount > 0
  const canAutoReadAdvance =
    !isManifestLoading && !manifestError && (isStripMode || pageSrc.length > 0)
  useReaderAutoRead({
    readMode: readerReadMode,
    pageStep,
    pageCount,
    currentIndex,
    controlsVisible: showReaderBottomBar,
    canAdvance: canAutoReadAdvance,
    stripScrollRef,
    onNextPage: goToNextPage
  })

  return (
    <main
      className="relative flex h-dvh overflow-hidden bg-neutral-950 text-neutral-50"
      onClick={toggleToolbar}
      onTouchMove={hideToolbar}
    >
      <ReaderTopBar
        fallbackReadId={comicId}
        title={title}
        chapter={chapter}
        isFetching={isFetching}
        visible={showReaderTopBar}
        onBack={goBack}
        onRetry={retry}
      />

      {isStripMode ? null : (
        <ReaderHotZones
          pageDirection={readerPageDirection}
          onPrevious={goToPreviousPageFromContent}
          onNext={goToNextPageOrChapterFromContent}
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
            key={comicId}
            containerRef={stripScrollRef}
            pages={pages}
            currentIndex={currentIndex}
            navigationCommand={navigationCommand}
            onCurrentIndexChange={observePage}
            onUserScroll={hideToolbar}
            onScrollPastEnd={goToNextChapter}
            onPrefetchThreshold={requestStripPrefetch}
            hasNextChapter={availableNextChapter != null}
          />
        ) : pageSrc.length > 0 ? (
          <ReaderImageWindow
            pages={pageWindow}
            currentIndex={currentIndex}
            pageCount={pageCount}
            doublePageMode={isDoublePageMode}
            pageDirection={readerPageDirection}
          />
        ) : (
          <ReaderError title="暂无图片" description="当前页没有可展示的图片" />
        )}
      </section>

      <ReaderBottomBar
        title={title}
        currentReadId={comicId}
        previousChapter={previousChapter}
        nextChapter={availableNextChapter}
        chapters={chapters}
        albumId={albumId}
        currentIndex={currentIndex}
        pageCount={pageCount}
        doublePageMode={isDoublePageMode}
        visible={showReaderBottomBar}
        onPageChange={goToPage}
      />
    </main>
  )
}
