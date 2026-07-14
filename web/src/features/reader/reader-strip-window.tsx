import { useVirtualizer, type Virtualizer } from '@tanstack/react-virtual'
import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  type RefObject,
  type WheelEvent
} from 'react'

import type { ComicReadManifestPage } from '@/lib/api/reader'
import { ReaderPageImage } from './reader-image'
import type { ReaderNavigationCommand } from './use-reader-session'

const STRIP_END_PADDING_PX = 128
const STRIP_END_SCROLL_THRESHOLD_PX = 24
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
  onScrollPastEnd
}: {
  containerRef: RefObject<HTMLDivElement | null>
  pages: ComicReadManifestPage[]
  currentIndex: number
  navigationCommand: ReaderNavigationCommand
  onCurrentIndexChange: (index: number) => void
  onUserScroll?: () => void
  onScrollPastEnd?: () => boolean
}) {
  const currentIndexRef = useRef(currentIndex)
  const navigationFrameRef = useRef<number | null>(null)
  const scrollFrameRef = useRef<number | null>(null)
  const lastNavigationIdRef = useRef<number | null>(null)
  const pendingTargetRef = useRef<number | null>(currentIndex)
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

    const targetIndex = Math.min(
      Math.max(currentIndex, 0),
      pages.length - 1
    )

    return targetIndex * estimatePageSize()
  }, [currentIndex, estimatePageSize, pages.length])
  const getPageKey = useCallback((index: number) => pages[index]?.path ?? index, [pages])
  const handleVirtualizerChange = useCallback(
    (virtualizer: Virtualizer<HTMLDivElement, HTMLElement>) => {
      const activeIndex = resolveActiveIndex(virtualizer, containerRef.current?.scrollTop)

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
    paddingEnd: STRIP_END_PADDING_PX,
    onChange: handleVirtualizerChange,
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
      navigationFrameRef.current = null
      virtualizer.scrollToIndex(targetIndex, { align: 'start', behavior: 'auto' })
    })
  }, [currentIndex, navigationCommand, pages.length, virtualizer])

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
    })
  }, [handleVirtualizerChange, onUserScroll, virtualizer])

  const handleWheel = useCallback(
    (event: WheelEvent<HTMLDivElement>) => {
      if (event.deltaY !== 0) {
        onUserScroll?.()
      }

      if (event.deltaY <= 0) {
        return
      }

      const container = event.currentTarget
      const maxScrollTop = Math.max(container.scrollHeight - container.clientHeight, 0)

      if (maxScrollTop - container.scrollTop > STRIP_END_SCROLL_THRESHOLD_PX) {
        return
      }

      if (onScrollPastEnd?.()) {
        event.preventDefault()
      }
    },
    [onScrollPastEnd, onUserScroll]
  )

  return (
    <div
      ref={containerRef}
      className="h-screen w-screen scrollbar-none overflow-y-auto overscroll-contain bg-neutral-950"
      onScroll={handleScroll}
      onTouchMove={onUserScroll}
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
              className="absolute top-0 left-0 flex min-h-[64vh] w-full justify-center bg-neutral-950"
              style={{ transform: `translateY(${virtualPage.start}px)` }}
            >
              <ReaderPageImage
                src={page.path}
                label={`第 ${virtualPage.index + 1} 页`}
                wrapperClassName="min-h-[64vh] w-full"
                imageClassName="block h-auto w-full object-contain"
                loading="eager"
                decoding={isActive ? 'sync' : 'async'}
                showLoadingIndicator={isActive}
              />
            </article>
          )
        })}
      </div>
    </div>
  )
}

function resolveActiveIndex(
  virtualizer: Virtualizer<HTMLDivElement, HTMLElement>,
  scrollOffset = virtualizer.scrollOffset ?? 0
) {
  const virtualPages = virtualizer.getVirtualItems()

  if (virtualPages.length === 0) {
    return null
  }

  const viewportHeight = virtualizer.scrollRect?.height ?? 0
  const trackingPoint =
    scrollOffset + viewportHeight * STRIP_TRACKING_VIEWPORT_RATIO
  const activePage = virtualPages.find(
    page => trackingPoint >= page.start && trackingPoint < page.end
  )

  if (activePage) {
    return activePage.index
  }

  return virtualPages.reduce((nearest, page) =>
    Math.abs(page.start - trackingPoint) < Math.abs(nearest.start - trackingPoint) ? page : nearest
  ).index
}

function getWindowWidth() {
  return typeof window === 'undefined' ? STRIP_MAX_CONTENT_WIDTH_PX : window.innerWidth
}

function getWindowHeight() {
  return typeof window === 'undefined' ? 800 : window.innerHeight
}
