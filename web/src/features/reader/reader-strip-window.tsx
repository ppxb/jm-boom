import { useVirtualizer, type Virtualizer } from '@tanstack/react-virtual'
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  type RefObject,
  type TouchEvent,
  type WheelEvent
} from 'react'

import type { ComicReadManifestPage } from '@/lib/api/reader'
import { READER } from '@/lib/constants'
import { ReaderPageImage } from './reader-image'
import type { ReaderNavigationCommand } from './use-reader-session'

const STRIP_END_PADDING_PX = 128
const STRIP_ESTIMATED_PAGE_RATIO = 1.45
const STRIP_MAX_CONTENT_WIDTH_PX = 1024
const STRIP_MIN_PAGE_VIEWPORT_RATIO = 0.64
const STRIP_OVERSCAN_PAGES = 3
const STRIP_TRACKING_VIEWPORT_RATIO = 0.25

export function ReaderStripWindow({
  containerRef,
  pages,
  currentIndex,
  navigationCommand,
  onCurrentIndexChange,
  onUserScroll,
  onScrollPastEnd,
  onPrefetchThreshold,
  hasNextChapter
}: {
  containerRef: RefObject<HTMLDivElement | null>
  pages: ComicReadManifestPage[]
  currentIndex: number
  navigationCommand: ReaderNavigationCommand
  onCurrentIndexChange: (index: number) => void
  onUserScroll?: () => void
  onScrollPastEnd?: () => boolean
  onPrefetchThreshold?: () => void
  hasNextChapter: boolean
}) {
  const currentIndexRef = useRef(currentIndex)
  const navigationFrameRef = useRef<number | null>(null)
  const scrollFrameRef = useRef<number | null>(null)
  const lastNavigationIdRef = useRef<number | null>(null)
  const pendingTargetRef = useRef<number | null>(currentIndex)
  const prefetchThresholdReportedRef = useRef(false)
  const endPullStartYRef = useRef<number | null>(null)
  const chapterAdvanceTriggeredRef = useRef(false)
  const endPadding = hasNextChapter ? STRIP_END_PADDING_PX : 0
  const estimatePageSize = useCallback(() => {
    const container = containerRef.current
    const viewportWidth = container?.clientWidth ?? getWindowWidth()
    const viewportHeight = container?.clientHeight ?? getWindowHeight()
    const contentWidth = Math.min(viewportWidth, STRIP_MAX_CONTENT_WIDTH_PX)

    return Math.max(
      viewportHeight * STRIP_MIN_PAGE_VIEWPORT_RATIO,
      contentWidth * STRIP_ESTIMATED_PAGE_RATIO
    )
  }, [containerRef])
  const initialOffset = useCallback(() => {
    if (pages.length <= 0) {
      return 0
    }

    const targetIndex = Math.min(Math.max(currentIndex, 0), pages.length - 1)

    return targetIndex * estimatePageSize()
  }, [currentIndex, estimatePageSize, pages.length])
  const getPageKey = useCallback((index: number) => pages[index]?.path ?? index, [pages])
  const handleVirtualizerChange = useCallback(
    (virtualizer: Virtualizer<HTMLDivElement, HTMLElement>) => {
      const activeIndex = resolveActiveIndex(virtualizer, containerRef.current)

      if (activeIndex == null) {
        return
      }

      const pendingTarget = pendingTargetRef.current

      if (pendingTarget != null) {
        if (activeIndex !== pendingTarget) {
          return
        }

        pendingTargetRef.current = null
      }

      if (activeIndex !== currentIndexRef.current) {
        currentIndexRef.current = activeIndex
        onCurrentIndexChange(activeIndex)
      }
    },
    [containerRef, onCurrentIndexChange]
  )
  const virtualizer = useVirtualizer<HTMLDivElement, HTMLElement>({
    count: pages.length,
    getScrollElement: () => containerRef.current,
    estimateSize: estimatePageSize,
    initialOffset,
    getItemKey: getPageKey,
    overscan: STRIP_OVERSCAN_PAGES,
    paddingEnd: endPadding,
    onChange: handleVirtualizerChange,
    measureElement: measurePageHeight,
    useAnimationFrameWithResizeObserver: true
  })
  const virtualPages = virtualizer.getVirtualItems()

  useEffect(() => {
    currentIndexRef.current = currentIndex
  }, [currentIndex])

  useLayoutEffect(() => {
    if (pages.length <= 0) {
      return
    }

    const isInitialNavigation = lastNavigationIdRef.current == null

    if (!isInitialNavigation && lastNavigationIdRef.current === navigationCommand.id) {
      return
    }

    const targetIndex = Math.min(
      Math.max(isInitialNavigation ? currentIndex : navigationCommand.targetIndex, 0),
      pages.length - 1
    )
    lastNavigationIdRef.current = navigationCommand.id
    pendingTargetRef.current = targetIndex
    currentIndexRef.current = targetIndex
    virtualizer.scrollToIndex(targetIndex, { align: 'start', behavior: 'auto' })

    if (navigationFrameRef.current !== null) {
      window.cancelAnimationFrame(navigationFrameRef.current)
    }

    navigationFrameRef.current = window.requestAnimationFrame(() => {
      virtualizer.scrollToIndex(targetIndex, { align: 'start', behavior: 'auto' })
      navigationFrameRef.current = window.requestAnimationFrame(() => {
        navigationFrameRef.current = null

        if (pendingTargetRef.current === targetIndex) {
          pendingTargetRef.current = null
          handleVirtualizerChange(virtualizer)
        }
      })
    })
  }, [currentIndex, handleVirtualizerChange, navigationCommand, pages.length, virtualizer])

  useEffect(
    () => () => {
      if (scrollFrameRef.current !== null) {
        window.cancelAnimationFrame(scrollFrameRef.current)
      }

      if (navigationFrameRef.current !== null) {
        window.cancelAnimationFrame(navigationFrameRef.current)
      }
    },
    []
  )

  const handleScroll = useCallback(() => {
    onUserScroll?.()

    if (scrollFrameRef.current !== null) {
      return
    }

    scrollFrameRef.current = window.requestAnimationFrame(() => {
      scrollFrameRef.current = null
      handleVirtualizerChange(virtualizer)

      const container = containerRef.current

      if (!container || prefetchThresholdReportedRef.current) {
        return
      }

      const maxScrollTop = Math.max(container.scrollHeight - container.clientHeight, 0)
      const progress = maxScrollTop > 0 ? container.scrollTop / maxScrollTop : 1

      if (progress >= READER.NEXT_CHAPTER_PREFETCH_PROGRESS) {
        prefetchThresholdReportedRef.current = true
        onPrefetchThreshold?.()
      }
    })
  }, [containerRef, handleVirtualizerChange, onPrefetchThreshold, onUserScroll, virtualizer])

  const handleWheel = useCallback(
    (event: WheelEvent<HTMLDivElement>) => {
      if (event.deltaY !== 0) {
        onUserScroll?.()
      }

      if (event.deltaY <= 0) {
        return
      }

      if (!hasNextChapter || !isStripScrollAtEnd(event.currentTarget)) {
        return
      }

      onScrollPastEnd?.()
    },
    [hasNextChapter, onScrollPastEnd, onUserScroll]
  )
  const handleTouchStart = useCallback(() => {
    endPullStartYRef.current = null
    chapterAdvanceTriggeredRef.current = false
  }, [])
  const handleTouchMove = useCallback(
    (event: TouchEvent<HTMLDivElement>) => {
      onUserScroll?.()

      if (!hasNextChapter || chapterAdvanceTriggeredRef.current) {
        return
      }

      const touch = event.touches[0]

      if (!touch) {
        return
      }

      if (!isStripScrollAtEnd(event.currentTarget)) {
        endPullStartYRef.current = null
        return
      }

      if (endPullStartYRef.current == null) {
        endPullStartYRef.current = touch.clientY
        return
      }

      if (!shouldAdvanceStripChapter(endPullStartYRef.current, touch.clientY)) {
        return
      }

      if (onScrollPastEnd?.()) {
        chapterAdvanceTriggeredRef.current = true
        endPullStartYRef.current = null
      }
    },
    [hasNextChapter, onScrollPastEnd, onUserScroll]
  )
  const handleTouchEnd = useCallback(() => {
    endPullStartYRef.current = null
    chapterAdvanceTriggeredRef.current = false
  }, [])

  return (
    <div
      ref={containerRef}
      data-scroll-restoration-id={READER.STRIP_SCROLL_RESTORATION_ID}
      className="h-dvh w-screen scrollbar-none overflow-y-auto overscroll-contain bg-neutral-950"
      onScroll={handleScroll}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
      onTouchCancel={handleTouchEnd}
      onWheel={handleWheel}
    >
      <div
        className="relative mx-auto w-full max-w-5xl bg-neutral-950"
        style={{ height: virtualizer.getTotalSize() }}
      >
        {virtualPages.map(virtualPage => {
          const page = pages[virtualPage.index]

          if (!page) {
            return null
          }

          const isActive = virtualPage.index === currentIndex

          return (
            <article
              key={virtualPage.key}
              ref={virtualizer.measureElement}
              data-index={virtualPage.index}
              data-reader-page-index={virtualPage.index}
              className="absolute top-0 left-0 flex w-full justify-center bg-neutral-950"
              style={{ transform: `translateY(${virtualPage.start}px)` }}
            >
              <ReaderPageImage
                src={page.path}
                label={`第 ${virtualPage.index + 1} 页`}
                wrapperClassName="w-full"
                placeholderClassName="min-h-[64vh]"
                imageClassName="block h-auto w-full object-contain"
                fitContainer
                loading={isActive ? 'eager' : 'lazy'}
                decoding={isActive ? 'sync' : 'async'}
                showLoadingIndicator={isActive}
              />
            </article>
          )
        })}
        {hasNextChapter ? (
          <div
            className="absolute top-0 left-0 flex w-full items-center justify-center text-sm text-neutral-400"
            style={{
              height: STRIP_END_PADDING_PX,
              transform: `translateY(${Math.max(
                virtualizer.getTotalSize() - STRIP_END_PADDING_PX,
                0
              )}px)`
            }}
          >
            滚动进入下一章
          </div>
        ) : null}
      </div>
    </div>
  )
}

function resolveActiveIndex(
  virtualizer: Virtualizer<HTMLDivElement, HTMLElement>,
  container: HTMLDivElement | null
) {
  const virtualPages = virtualizer.getVirtualItems()

  if (virtualPages.length === 0) {
    return null
  }

  const scrollOffset = container?.scrollTop ?? virtualizer.scrollOffset ?? 0
  const viewportHeight = container?.clientHeight ?? virtualizer.scrollRect?.height ?? 0
  const trackingPoint = scrollOffset + viewportHeight * STRIP_TRACKING_VIEWPORT_RATIO
  const activePage = virtualPages.find(
    page => trackingPoint >= page.start && trackingPoint < page.end
  )
  const candidateIndex = activePage
    ? activePage.index
    : virtualPages.reduce((nearest, page) =>
        Math.abs(page.start - trackingPoint) < Math.abs(nearest.start - trackingPoint)
          ? page
          : nearest
      ).index

  if (!container) {
    return candidateIndex
  }

  return resolveStripTrackedIndex({
    candidateIndex,
    pageCount: virtualizer.options.count,
    scrollTop: container.scrollTop,
    scrollHeight: container.scrollHeight,
    clientHeight: container.clientHeight
  })
}

export function resolveStripTrackedIndex({
  candidateIndex,
  pageCount,
  scrollTop,
  scrollHeight,
  clientHeight
}: {
  candidateIndex: number
  pageCount: number
  scrollTop: number
  scrollHeight: number
  clientHeight: number
}) {
  if (pageCount <= 0) {
    return null
  }

  if (isStripScrollAtEnd({ scrollTop, scrollHeight, clientHeight })) {
    return pageCount - 1
  }

  return Math.min(Math.max(candidateIndex, 0), pageCount - 1)
}

export function shouldAdvanceStripChapter(startY: number, currentY: number) {
  return startY - currentY >= READER.STRIP_NEXT_CHAPTER_SWIPE_DISTANCE
}

function isStripScrollAtEnd({
  scrollTop,
  scrollHeight,
  clientHeight
}: {
  scrollTop: number
  scrollHeight: number
  clientHeight: number
}) {
  const maxScrollTop = Math.max(scrollHeight - clientHeight, 0)

  return maxScrollTop - scrollTop <= READER.STRIP_SCROLL_THRESHOLD
}

// TanStack's default measurement rounds the observed height to the nearest
// pixel. Rounding up places the next page a fraction below the previous page's
// real bottom, exposing the black backdrop as a hairline between pages. Flooring
// instead lets pages overlap by a sub-pixel (hidden by overflow) so the strip
// stays seamless.
function measurePageHeight(
  element: Element,
  _entry: ResizeObserverEntry | undefined,
  instance: Virtualizer<HTMLDivElement, HTMLElement>
) {
  const height = element.getBoundingClientRect().height
  if (height > 0) {
    return Math.floor(height)
  }

  return (element as HTMLElement).offsetHeight || instance.options.estimateSize(0)
}

function getWindowWidth() {
  return typeof window === 'undefined' ? STRIP_MAX_CONTENT_WIDTH_PX : window.innerWidth
}

function getWindowHeight() {
  return typeof window === 'undefined' ? 800 : window.innerHeight
}
