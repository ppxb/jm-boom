import { useNavigate, useRouter } from '@tanstack/react-router'
import { useCallback, useEffect } from 'react'

import { ReaderBottomBar, ReaderTopBar } from './reader-bars'
import { ReaderHotZones } from './reader-hot-zones'
import { ReaderImage } from './reader-image'
import { ReaderError, ReaderLoading } from './reader-state'
import type { ReaderSearch } from './types'
import { useReaderChapterInfo } from './use-reader-chapter-info'
import { useReaderKeyboardNavigation } from './use-reader-keyboard-navigation'
import { useReaderPages } from './use-reader-pages'
import { useReaderToolbarVisibility } from './use-reader-toolbar-visibility'
import { useReadingHistoryStore } from '@/stores/reading-history-store'

const DEFAULT_CHAPTER_TITLE = '正文'

export function ReaderPage({ comicId, search }: { comicId: string; search: ReaderSearch }) {
  const navigate = useNavigate()
  const router = useRouter()
  const upsertReadingHistory = useReadingHistoryStore(state => state.upsert)
  const {
    isVisible: isToolbarVisible,
    toggle: toggleToolbar,
    hide: hideToolbar
  } = useReaderToolbarVisibility()
  const initialPageIndex = Number.parseInt(search.pageIndex ?? '', 10)
  const { albumId, title, author, coverUrl, chapter, nextChapter } = useReaderChapterInfo({
    comicId,
    search
  })
  const {
    currentIndex,
    pageCount,
    pageSrc,
    isLastPage,
    isManifestLoading,
    manifestError,
    isPageLoading,
    pageError,
    isFetching,
    goToPreviousPage,
    goToNextPage,
    retry
  } = useReaderPages(comicId, Number.isNaN(initialPageIndex) ? 0 : initialPageIndex)

  useEffect(() => {
    if (!comicId || pageCount <= 0) {
      return
    }

    const historyComicId = albumId || comicId
    const historyTitle = title || `JM ${historyComicId}`
    const historyChapter = chapter || DEFAULT_CHAPTER_TITLE

    upsertReadingHistory({
      comicId: historyComicId,
      albumId,
      title: historyTitle,
      author,
      coverUrl,
      chapterId: comicId,
      chapterTitle: historyChapter,
      pageIndex: currentIndex,
      pageCount
    })
  }, [
    author,
    chapter,
    comicId,
    coverUrl,
    currentIndex,
    pageCount,
    albumId,
    title,
    upsertReadingHistory
  ])
  const goBack = useCallback(() => {
    if (window.history.length > 1) {
      router.history.back()
      return
    }

    if (search.fromDetail === '1' && albumId.length > 0) {
      void navigate({ to: '/comic/$comicId', params: { comicId: albumId }, replace: true })
      return
    }

    void navigate({ to: '/' })
  }, [albumId, navigate, router, search.fromDetail])

  useReaderKeyboardNavigation({
    onPrevious: goToPreviousPage,
    onNext: goToNextPage,
    onBack: goBack,
    onNavigate: hideToolbar
  })

  return (
    <main
      className="relative flex h-screen overflow-hidden bg-neutral-950 text-neutral-50"
      onClick={toggleToolbar}
    >
      <ReaderTopBar
        fallbackReadId={comicId}
        title={title}
        chapter={chapter}
        isFetching={isFetching}
        visible={isToolbarVisible}
        onBack={goBack}
        onRetry={retry}
      />

      <ReaderHotZones onPrevious={goToPreviousPage} onNext={goToNextPage} />

      <section className="flex min-w-0 flex-1 items-center justify-center">
        {isManifestLoading ? (
          <ReaderLoading label="正在加载阅读信息" />
        ) : manifestError ? (
          <ReaderError title="阅读信息加载失败" description={manifestError.message} />
        ) : isPageLoading ? (
          <ReaderLoading label="正在准备图片" />
        ) : pageError ? (
          <ReaderError title="图片加载失败" description={pageError.message} />
        ) : pageSrc.length > 0 ? (
          <ReaderImage src={pageSrc} />
        ) : (
          <ReaderError title="暂无图片" description="当前页没有可展示的图片" />
        )}
      </section>

      <ReaderBottomBar
        title={title}
        nextChapter={nextChapter}
        albumId={albumId}
        currentIndex={currentIndex}
        pageCount={pageCount}
        showNextChapter={isLastPage && nextChapter != null}
        visible={isToolbarVisible}
      />
    </main>
  )
}
