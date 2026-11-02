import { useVirtualizer, type Virtualizer } from '@tanstack/react-virtual'
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, type RefObject } from 'react'

import { READER } from '@/lib/constants'
import { ReaderChapterBoundary } from './reader-chapter-boundary'
import { ReaderPageImage } from './reader-image'
import type { ReaderBoundary, ReaderChapterItem, ReaderPageItem } from './types'
import type { ReaderNavigationCommand } from './use-reader-session'

type StripItem = {
  key: string
  pageIndex: number
  transition?: {
    currentChapter: ReaderChapterItem
    boundary: ReaderBoundary
    previous?: boolean
    edge?: -1 | 1
  }
}
type Anchor = { key: string; offset: number }
const TRANSITION_ESTIMATED_HEIGHT = 240

export function ReaderStripWindow({
  containerRef,
  pages,
  currentIndex,
  navigationCommand,
  onCurrentIndexChange,
  onVisibleRangeChange,
  onUserScroll,
  previousBoundary,
  nextBoundary,
  onLoadBoundary,
  onPageLoaded,
  onPageReady,
  chapterTitles
}: {
  containerRef: RefObject<HTMLDivElement | null>
  pages: ReaderPageItem[]
  currentIndex: number
  navigationCommand: ReaderNavigationCommand
  onCurrentIndexChange: (index: number) => void
  onVisibleRangeChange: (first: number, last: number) => void
  onUserScroll: () => void
  previousBoundary: ReaderBoundary
  nextBoundary: ReaderBoundary
  onLoadBoundary: (direction: -1 | 1) => void
  onPageLoaded: (page: ReaderPageItem, image: HTMLImageElement) => void
  onPageReady: (key: string, ready: boolean) => void
  chapterTitles: Map<string, string>
}) {
  const items = useMemo(
    () => createStripItems(pages, chapterTitles, previousBoundary, nextBoundary),
    [chapterTitles, pages, previousBoundary, nextBoundary]
  )
  const committedItems = useRef(items)
  const anchor = useRef<Anchor | null>(null)
  const lastNavigation = useRef<number | null>(null)
  const navigationTarget = useRef<string | null>(null)
  const frame = useRef<number | null>(null)
  const scrollFrame = useRef<number | null>(null)
  const previousFirstKey = useRef(items[0]?.key)
  const estimateSize = useCallback(
    (index: number) => {
      const item = items[index]
      if (item.transition) return TRANSITION_ESTIMATED_HEIGHT
      const page = pages[item.pageIndex]
      const width = Math.min(containerRef.current?.clientWidth || window.innerWidth, 1024)
      const height = containerRef.current?.clientHeight || window.innerHeight
      return page.width && page.height
        ? Math.floor((width * page.height) / page.width)
        : Math.max(height * 0.64, width * 1.45)
    },
    [containerRef, items, pages]
  )
  const getItemKey = useCallback((index: number) => items[index].key, [items])
  const updateVisible = useCallback(
    (instance: Virtualizer<HTMLDivElement, HTMLElement>, trackPage = false) => {
      const container = containerRef.current
      if (!container || committedItems.current !== items || navigationTarget.current) return
      const top = container.scrollTop
      const bottom = top + container.clientHeight
      const visible = instance
        .getVirtualItems()
        .filter(item => item.end > top && item.start < bottom)
      if (!visible.length) return
      anchor.current = { key: String(visible[0].key), offset: top - visible[0].start }
      const visiblePages = visible.filter(item => !items[item.index]?.transition)
      const first = visiblePages[0] ?? visible[0]
      const last = visiblePages[visiblePages.length - 1] ?? visible[visible.length - 1]
      onVisibleRangeChange(items[first.index].pageIndex, items[last.index].pageIndex)
      if (!trackPage) return
      const trackingPoint = top + container.clientHeight * 0.25
      const tracked = visible.find(item => trackingPoint >= item.start && trackingPoint < item.end)
      if (tracked && items[tracked.index].transition) return
      const candidate = tracked ?? first
      if (items[candidate.index].transition) return
      const candidateIndex = items[candidate.index].pageIndex
      onCurrentIndexChange(
        resolveStripTrackedIndex({
          candidateIndex,
          pageCount: pages.length,
          scrollTop: top,
          scrollHeight: container.scrollHeight,
          clientHeight: container.clientHeight
        }) ?? candidateIndex
      )
    },
    [containerRef, items, onCurrentIndexChange, onVisibleRangeChange, pages.length]
  )
  const latestUpdate = useRef(updateVisible)
  useLayoutEffect(() => {
    latestUpdate.current = updateVisible
  }, [updateVisible])
  const virtualizer = useVirtualizer<HTMLDivElement, HTMLElement>({
    count: items.length,
    getScrollElement: () => containerRef.current,
    estimateSize,
    getItemKey,
    overscan: 3,
    onChange: instance => updateVisible(instance),
    measureElement: element =>
      Math.floor(element.getBoundingClientRect().height) || element.offsetHeight,
    useAnimationFrameWithResizeObserver: true
  })
  const virtualItems = virtualizer.getVirtualItems()

  useLayoutEffect(() => {
    const initial = lastNavigation.current == null
    const newCommand = lastNavigation.current !== navigationCommand.id
    const commanded = initial || newCommand || navigationTarget.current != null
    const changedBeforeViewport = previousFirstKey.current !== items[0]?.key
    committedItems.current = items
    previousFirstKey.current = items[0]?.key
    // The virtualizer handles image height changes; list edits are restored by stable page identity.
    virtualizer.shouldAdjustScrollPositionOnItemSizeChange = item =>
      item.start < (containerRef.current?.scrollTop ?? 0)
    if (commanded) {
      const key = initial
        ? pages[currentIndex]?.key
        : newCommand
          ? navigationCommand.targetKey
          : navigationTarget.current
      const index = items.findIndex(item => item.key === key)
      if (index < 0) return
      lastNavigation.current = navigationCommand.id
      navigationTarget.current = key!
      virtualizer.scrollToIndex(index, { align: 'start', behavior: 'auto' })
      if (frame.current != null) window.cancelAnimationFrame(frame.current)
      frame.current = window.requestAnimationFrame(() => {
        virtualizer.scrollToIndex(index, { align: 'start', behavior: 'auto' })
        frame.current = window.requestAnimationFrame(() => {
          frame.current = null
          navigationTarget.current = null
          updateVisible(virtualizer)
        })
      })
    } else if (changedBeforeViewport && anchor.current) {
      const offset = getStripAnchorOffset(anchor.current, virtualizer.measurementsCache)
      if (offset != null) virtualizer.scrollToOffset(offset, { behavior: 'auto' })
    }
  }, [containerRef, currentIndex, items, navigationCommand, pages, updateVisible, virtualizer])

  useEffect(
    () => () => {
      if (frame.current != null) window.cancelAnimationFrame(frame.current)
      if (scrollFrame.current != null) window.cancelAnimationFrame(scrollFrame.current)
    },
    []
  )

  return (
    <div
      ref={containerRef}
      data-scroll-restoration-id={READER.STRIP_SCROLL_RESTORATION_ID}
      className="h-dvh w-screen scrollbar-none overflow-y-auto overscroll-contain bg-neutral-950"
      style={{ overflowAnchor: 'none' }}
      onScroll={() => {
        onUserScroll()
        if (scrollFrame.current != null) return
        scrollFrame.current = window.requestAnimationFrame(() => {
          scrollFrame.current = null
          latestUpdate.current(virtualizer, true)
        })
      }}
      onTouchMove={onUserScroll}
      onWheel={onUserScroll}
    >
      <div
        className="relative mx-auto w-full max-w-5xl bg-neutral-950"
        style={{ height: virtualizer.getTotalSize() }}
      >
        {virtualItems.map(item => {
          const row = items[item.index]
          const page = pages[row.pageIndex]
          return (
            <article
              key={item.key}
              ref={virtualizer.measureElement}
              data-index={item.index}
              data-reader-page-key={row.transition ? undefined : page.key}
              data-reader-transition-key={row.transition ? row.key : undefined}
              className="absolute top-0 left-0 w-full bg-neutral-950"
              style={{ transform: 'translateY(' + item.start + 'px)' }}
            >
              {row.transition ? (
                <ReaderChapterBoundary
                  currentChapter={row.transition.currentChapter}
                  boundary={row.transition.boundary}
                  previous={row.transition.previous}
                  onRetry={
                    row.transition.edge ? () => onLoadBoundary(row.transition!.edge!) : undefined
                  }
                />
              ) : (
                <ReaderPageImage
                  src={page.path}
                  label={'第 ' + (page.localIndex + 1) + ' 页'}
                  width={page.width}
                  height={page.height}
                  wrapperClassName="w-full"
                  placeholderClassName="min-h-[64vh]"
                  imageClassName="block h-auto w-full object-contain"
                  fitContainer
                  loading="eager"
                  decoding="async"
                  showLoadingIndicator={row.pageIndex === currentIndex}
                  onLoad={image => onPageLoaded(page, image)}
                  onReadyChange={ready => onPageReady(page.key, ready)}
                />
              )}
            </article>
          )
        })}
      </div>
    </div>
  )
}

export function createStripItems(
  pages: ReaderPageItem[],
  chapterTitles: Map<string, string>,
  previousBoundary: ReaderBoundary,
  nextBoundary: ReaderBoundary
): StripItem[] {
  if (!pages.length) return []
  const chapter = (id: string): ReaderChapterItem => ({
    id,
    title: chapterTitles.get(id) || '正文'
  })
  const result: StripItem[] = []
  const firstChapter = chapter(pages[0].chapterId)
  if (previousBoundary.chapter) {
    result.push({
      key: 'chapter:' + previousBoundary.chapter.id + '>' + firstChapter.id,
      pageIndex: 0,
      transition: {
        currentChapter: firstChapter,
        boundary: previousBoundary,
        previous: true,
        edge: -1
      }
    })
  }
  pages.forEach((page, pageIndex) => {
    const previous = pages[pageIndex - 1]
    if (previous && previous.chapterId !== page.chapterId) {
      result.push({
        key: 'chapter:' + previous.chapterId + '>' + page.chapterId,
        pageIndex,
        transition: {
          currentChapter: chapter(previous.chapterId),
          boundary: { chapter: chapter(page.chapterId), loading: false, error: null }
        }
      })
    }
    result.push({ key: page.key, pageIndex })
  })
  const lastChapter = chapter(pages[pages.length - 1].chapterId)
  result.push({
    key: 'chapter:' + lastChapter.id + '>' + (nextBoundary.chapter?.id ?? 'end'),
    pageIndex: pages.length - 1,
    transition: { currentChapter: lastChapter, boundary: nextBoundary, edge: 1 }
  })
  return result
}

export function getStripAnchorOffset(
  anchor: Anchor,
  measurements: Array<{ key: string | number | bigint; start: number }>
) {
  const page = measurements.find(item => String(item.key) === anchor.key)
  return page ? Math.max(0, page.start + anchor.offset) : null
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
  if (pageCount <= 0) return null
  const maxScrollTop = Math.max(scrollHeight - clientHeight, 0)
  if (maxScrollTop > 0 && maxScrollTop - scrollTop <= READER.STRIP_SCROLL_THRESHOLD)
    return pageCount - 1
  return Math.min(Math.max(candidateIndex, 0), pageCount - 1)
}
