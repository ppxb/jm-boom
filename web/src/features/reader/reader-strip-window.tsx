import { useQuery } from '@tanstack/react-query'
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type RefObject,
  type WheelEvent
} from 'react'

import { Button } from '@/components/ui/button'
import { CACHE } from '@/lib/constants'
import { ReaderPageImage } from './reader-image'
import { ReaderLoading } from './reader-state'
import type { ReaderPageQueryKeyFactory, ReaderPageRequester } from './use-reader-page-query'

const DEFAULT_PAGE_HEIGHT_RATIO = 1.45
const STRIP_END_PADDING_PX = 128
const STRIP_END_SCROLL_THRESHOLD_PX = 24
const STRIP_MAX_CONTENT_WIDTH_PX = 1024
const STRIP_OVERSCAN_VIEWPORTS = 1.5
const SIZE_CACHE_PREFIX = 'jm-boom-reader-strip-sizes:'

type PendingNavigation = {
  requestId: number
  targetIndex: number
}

type ViewportState = {
  height: number
  scrollTop: number
}

export function ReaderStripWindow({
  cacheKey,
  containerRef,
  pageCount,
  currentIndex,
  navigationRequestId,
  pageQueryKey,
  requestPage,
  onCurrentIndexChange,
  onUserScroll,
  onScrollPastEnd
}: {
  cacheKey: string
  containerRef: RefObject<HTMLDivElement | null>
  pageCount: number
  currentIndex: number
  navigationRequestId: number
  pageQueryKey: ReaderPageQueryKeyFactory
  requestPage: ReaderPageRequester
  onCurrentIndexChange: (index: number) => void
  onUserScroll?: () => void
  onScrollPastEnd?: () => boolean
}) {
  const [pageHeightRatios, setPageHeightRatios] = useState<Record<number, number>>(() =>
    readPageHeightRatios(cacheKey)
  )
  const [contentWidth, setContentWidth] = useState(getInitialContentWidth)
  const [viewport, setViewport] = useState<ViewportState>(getInitialViewport)
  const frameRef = useRef<number | null>(null)
  const navigationFrameRef = useRef<number | null>(null)
  const lastNavigationRequestRef = useRef<number | null>(null)
  const pendingNavigationRef = useRef<PendingNavigation | null>(null)
  const programmaticScrollTopRef = useRef<number | null>(null)
  const currentIndexRef = useRef(currentIndex)
  const pageHeightRatiosRef = useRef(pageHeightRatios)
  const contentWidthRef = useRef(contentWidth)

  const effectiveContentWidth = Math.max(contentWidth, 1)
  const pageHeights = useMemo(
    () =>
      Array.from({ length: pageCount }, (_, index) =>
        Math.max(1, effectiveContentWidth * (pageHeightRatios[index] ?? DEFAULT_PAGE_HEIGHT_RATIO))
      ),
    [effectiveContentWidth, pageCount, pageHeightRatios]
  )
  const pageOffsets = useMemo(() => createPageOffsets(pageHeights), [pageHeights])
  const pageOffsetsRef = useRef(pageOffsets)
  const totalHeight = (pageOffsets[pageCount] ?? 0) + STRIP_END_PADDING_PX
  const viewportHeight = Math.max(viewport.height, 1)
  const overscan = viewportHeight * STRIP_OVERSCAN_VIEWPORTS
  const visibleStartIndex = findPageAtOffset(pageOffsets, viewport.scrollTop, pageCount)
  const visibleEndIndex = findPageAtOffset(
    pageOffsets,
    viewport.scrollTop + viewportHeight,
    pageCount
  )
  const activeIndex = findPageAtOffset(
    pageOffsets,
    viewport.scrollTop + viewportHeight / 2,
    pageCount
  )
  const renderStartIndex = findPageAtOffset(
    pageOffsets,
    Math.max(0, viewport.scrollTop - overscan),
    pageCount
  )
  const renderEndIndex = findPageAtOffset(
    pageOffsets,
    viewport.scrollTop + viewportHeight + overscan,
    pageCount
  )
  const virtualIndexes = useMemo(() => {
    const indexes = new Set<number>()

    for (let index = renderStartIndex; index <= renderEndIndex; index += 1) {
      indexes.add(index)
    }

    if (currentIndex >= 0 && currentIndex < pageCount) {
      indexes.add(currentIndex)
    }

    return [...indexes].sort((left, right) => left - right)
  }, [currentIndex, pageCount, renderEndIndex, renderStartIndex])

  useEffect(() => {
    currentIndexRef.current = currentIndex
  }, [currentIndex])

  useEffect(() => {
    pageHeightRatiosRef.current = pageHeightRatios
  }, [pageHeightRatios])

  useEffect(() => {
    pageOffsetsRef.current = pageOffsets
  }, [pageOffsets])

  useEffect(() => {
    const container = containerRef.current

    if (!container) {
      return
    }

    const updateSize = () => {
      const nextContentWidth = Math.min(container.clientWidth, STRIP_MAX_CONTENT_WIDTH_PX)
      const previousContentWidth = contentWidthRef.current

      if (previousContentWidth > 0 && nextContentWidth !== previousContentWidth) {
        const nextScrollTop = container.scrollTop * (nextContentWidth / previousContentWidth)
        programmaticScrollTopRef.current = nextScrollTop
        container.scrollTop = nextScrollTop
      }

      contentWidthRef.current = nextContentWidth
      setContentWidth(nextContentWidth)
      setViewport({ height: container.clientHeight, scrollTop: container.scrollTop })
    }

    updateSize()

    if (typeof ResizeObserver === 'undefined') {
      window.addEventListener('resize', updateSize)
      return () => window.removeEventListener('resize', updateSize)
    }

    const observer = new ResizeObserver(updateSize)
    observer.observe(container)

    return () => observer.disconnect()
  }, [containerRef])

  useEffect(() => {
    if (lastNavigationRequestRef.current === navigationRequestId || pageCount <= 0) {
      return
    }

    const container = containerRef.current

    if (!container) {
      return
    }

    const targetIndex = Math.min(Math.max(currentIndex, 0), pageCount - 1)
    const targetTop = pageOffsetsRef.current[targetIndex] ?? 0
    lastNavigationRequestRef.current = navigationRequestId
    pendingNavigationRef.current = { requestId: navigationRequestId, targetIndex }
    container.scrollTo({ top: targetTop, behavior: 'auto' })
    setViewport({ height: container.clientHeight, scrollTop: targetTop })

    if (navigationFrameRef.current !== null) {
      window.cancelAnimationFrame(navigationFrameRef.current)
    }

    navigationFrameRef.current = window.requestAnimationFrame(() => {
      navigationFrameRef.current = window.requestAnimationFrame(() => {
        const pending = pendingNavigationRef.current

        if (pending?.requestId === navigationRequestId) {
          pendingNavigationRef.current = null
        }

        navigationFrameRef.current = null
      })
    })
  }, [containerRef, currentIndex, navigationRequestId, pageCount])

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      writePageHeightRatios(cacheKey, pageHeightRatios)
    }, 300)

    return () => window.clearTimeout(timeout)
  }, [cacheKey, pageHeightRatios])

  useEffect(
    () => () => {
      if (frameRef.current !== null) {
        window.cancelAnimationFrame(frameRef.current)
      }

      if (navigationFrameRef.current !== null) {
        window.cancelAnimationFrame(navigationFrameRef.current)
      }
    },
    []
  )

  const resolveCurrentIndex = useCallback(
    (container: HTMLDivElement) => {
      if (pendingNavigationRef.current || pageCount <= 0) {
        return
      }

      const viewportCenter = container.scrollTop + container.clientHeight / 2
      const nextIndex = findPageAtOffset(pageOffsetsRef.current, viewportCenter, pageCount)

      if (nextIndex !== currentIndexRef.current) {
        onCurrentIndexChange(nextIndex)
      }
    },
    [onCurrentIndexChange, pageCount]
  )

  const handleScroll = useCallback(() => {
    if (frameRef.current !== null) {
      return
    }

    frameRef.current = window.requestAnimationFrame(() => {
      frameRef.current = null
      const container = containerRef.current

      if (!container) {
        return
      }

      setViewport({ height: container.clientHeight, scrollTop: container.scrollTop })

      const programmaticScrollTop = programmaticScrollTopRef.current
      const isProgrammaticScroll =
        programmaticScrollTop !== null &&
        Math.abs(programmaticScrollTop - container.scrollTop) < 1

      if (isProgrammaticScroll) {
        programmaticScrollTopRef.current = null
      }

      if (!pendingNavigationRef.current && !isProgrammaticScroll) {
        onUserScroll?.()
      }

      resolveCurrentIndex(container)
    })
  }, [containerRef, onUserScroll, resolveCurrentIndex])

  const handlePageSize = useCallback(
    (index: number, image: HTMLImageElement) => {
      if (image.naturalWidth <= 0 || image.naturalHeight <= 0 || contentWidth <= 0) {
        return
      }

      const nextRatio = image.naturalHeight / image.naturalWidth
      const previousRatio =
        pageHeightRatiosRef.current[index] ?? DEFAULT_PAGE_HEIGHT_RATIO

      if (Math.abs(nextRatio - previousRatio) < 0.001) {
        return
      }

      const container = containerRef.current
      const anchorIndex = pendingNavigationRef.current?.targetIndex ?? currentIndexRef.current

      if (container && index < anchorIndex) {
        const nextScrollTop = container.scrollTop + contentWidth * (nextRatio - previousRatio)
        programmaticScrollTopRef.current = nextScrollTop
        container.scrollTop = nextScrollTop
      }

      setPageHeightRatios(current => ({ ...current, [index]: nextRatio }))
    },
    [containerRef, contentWidth]
  )

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
        style={{ height: totalHeight }}
      >
        {virtualIndexes.map(index => (
          <ReaderStripImage
            key={index}
            index={index}
            top={pageOffsets[index] ?? 0}
            height={pageHeights[index] ?? 1}
            isVisible={index >= visibleStartIndex && index <= visibleEndIndex}
            isActive={index === activeIndex}
            pageQueryKey={pageQueryKey}
            requestPage={requestPage}
            onPageSize={handlePageSize}
          />
        ))}
      </div>
    </div>
  )
}

function ReaderStripImage({
  index,
  top,
  height,
  isVisible,
  isActive,
  pageQueryKey,
  requestPage,
  onPageSize
}: {
  index: number
  top: number
  height: number
  isVisible: boolean
  isActive: boolean
  pageQueryKey: ReaderPageQueryKeyFactory
  requestPage: ReaderPageRequester
  onPageSize: (index: number, image: HTMLImageElement) => void
}) {
  const page = useQuery({
    queryKey: pageQueryKey(index),
    queryFn: ({ signal }) => requestPage(index, isVisible ? 'visible' : 'prefetch', signal),
    staleTime: CACHE.READER_STALE_TIME,
    gcTime: CACHE.READER_GC_TIME,
    retry: false,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const src = page.data?.index === index ? page.data.path : ''

  return (
    <article
      className="absolute inset-x-0 flex w-full justify-center bg-neutral-950"
      style={{ height, top }}
      data-reader-page-index={index}
    >
      {src ? (
        <ReaderPageImage
          src={src}
          label={`第 ${index + 1} 页`}
          wrapperClassName="h-full w-full"
          imageClassName="block h-auto w-full object-contain"
          loading="eager"
          decoding={isVisible ? 'sync' : 'async'}
          showLoadingIndicator={isActive}
          loadingIndicatorClassName="!fixed inset-0 z-10"
          onLoad={image => onPageSize(index, image)}
        />
      ) : page.isError ? (
        <div className="flex h-full w-full flex-col items-center justify-center gap-3 px-6 text-center text-sm text-neutral-300">
          <div>
            <div className="font-medium text-neutral-100">第 {index + 1} 张加载失败</div>
            <div className="mt-1 text-xs text-neutral-500">{page.error.message}</div>
          </div>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="border-white/15 bg-white/5 text-neutral-100 hover:bg-white/10 hover:text-neutral-50"
            onClick={event => {
              event.stopPropagation()
              void page.refetch()
            }}
          >
            重试
          </Button>
        </div>
      ) : (
        isActive ? (
          <ReaderLoading
            label={`正在加载第 ${index + 1} 页`}
            className="fixed inset-0 z-10"
          />
        ) : null
      )}
    </article>
  )
}

function createPageOffsets(pageHeights: number[]) {
  const offsets = Array.from({ length: pageHeights.length + 1 }, () => 0)
  offsets[0] = 0

  for (let index = 0; index < pageHeights.length; index += 1) {
    offsets[index + 1] = offsets[index] + pageHeights[index]
  }

  return offsets
}

function findPageAtOffset(pageOffsets: number[], offset: number, pageCount: number) {
  if (pageCount <= 0) {
    return 0
  }

  let low = 0
  let high = pageCount - 1

  while (low <= high) {
    const middle = Math.floor((low + high) / 2)
    const pageStart = pageOffsets[middle] ?? 0
    const pageEnd = pageOffsets[middle + 1] ?? pageStart

    if (offset < pageStart) {
      high = middle - 1
    } else if (offset >= pageEnd) {
      low = middle + 1
    } else {
      return middle
    }
  }

  return Math.min(Math.max(low, 0), pageCount - 1)
}

function readPageHeightRatios(cacheKey: string) {
  if (typeof localStorage === 'undefined') {
    return {}
  }

  try {
    const value = localStorage.getItem(`${SIZE_CACHE_PREFIX}${cacheKey}`)
    const parsed = value ? (JSON.parse(value) as Record<number, number>) : {}

    return Object.fromEntries(
      Object.entries(parsed).filter(([, ratio]) => Number.isFinite(ratio) && ratio > 0 && ratio < 100)
    )
  } catch {
    return {}
  }
}

function getInitialContentWidth() {
  if (typeof window === 'undefined') {
    return STRIP_MAX_CONTENT_WIDTH_PX
  }

  return Math.min(window.innerWidth, STRIP_MAX_CONTENT_WIDTH_PX)
}

function getInitialViewport(): ViewportState {
  return {
    height: typeof window === 'undefined' ? 800 : window.innerHeight,
    scrollTop: 0
  }
}

function writePageHeightRatios(cacheKey: string, ratios: Record<number, number>) {
  if (typeof localStorage === 'undefined') {
    return
  }

  try {
    localStorage.setItem(`${SIZE_CACHE_PREFIX}${cacheKey}`, JSON.stringify(ratios))
  } catch {
    // Size persistence is an optimization; reading must keep working when storage is unavailable.
  }
}
