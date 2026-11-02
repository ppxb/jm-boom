import { useQueries, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { getComicReadManifest } from '@/lib/api/reader'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { readerSpread } from './reader-page-layout'
import type { ReaderBoundary, ReaderChapterItem, ReaderPageItem, ReaderWindowPage } from './types'

const EMPTY_CHAPTERS: ReaderChapterItem[] = []
type Position = { readId: string; index: number }
type Jump = Position & { loading: boolean; error: string | null }
type ImageSize = { width: number; height: number }

export type ReaderNavigationCommand = {
  id: number
  targetIndex: number
  targetKey?: string
}

function manifestOptions(readId: string) {
  return {
    queryKey: queryKeys.readerManifest(readId),
    queryFn: () => getComicReadManifest({ readId }),
    staleTime: CACHE.READER_STALE_TIME,
    gcTime: CACHE.READER_GC_TIME,
    retry: false as const
  }
}

export function useReaderSession({
  comicId,
  initialIndex = 0,
  pageStep = 1,
  paged = false,
  chapters = EMPTY_CHAPTERS,
  onLocationChange
}: {
  comicId: string
  initialIndex?: number
  pageStep?: number
  paged?: boolean
  chapters?: ReaderChapterItem[]
  onLocationChange?: (readId: string, index: number) => void | Promise<void>
}) {
  const queryClient = useQueryClient()
  const [position, setPosition] = useState<Position>({
    readId: comicId,
    index: normalizeIndex(initialIndex)
  })
  const [visibleChapters, setVisibleChapters] = useState<string[]>([])
  const [extendedChapter, setExtendedChapter] = useState<string | null>(null)
  const [jump, setJump] = useState<Jump | null>(null)
  const [transitionDirection, setTransitionDirection] = useState<-1 | 1 | null>(null)
  const [imageSizes, setImageSizes] = useState(() => new Map<string, ImageSize>())
  const [readyPages, setReadyPages] = useState(() => new Set<string>())
  const [navigationCommand, setNavigationCommand] = useState<ReaderNavigationCommand>({
    id: 0,
    targetIndex: normalizeIndex(initialIndex)
  })
  const navigationEpoch = useRef(0)
  const jumpRef = useRef(jump)
  const chapterList = useMemo(
    () =>
      chapters.some(chapter => chapter.id === position.readId)
        ? chapters
        : [{ id: position.readId, title: '' }],
    [chapters, position.readId]
  )
  const windowIds = useMemo(() => {
    const active = chapterList.findIndex(chapter => chapter.id === position.readId)
    const retained = [...visibleChapters, extendedChapter]
      .map(id => chapterList.findIndex(chapter => chapter.id === id))
      .filter(index => index >= 0)
    const first = Math.max(0, Math.min(active, ...retained) - 1)
    const last = Math.min(chapterList.length - 1, Math.max(active, ...retained) + 1)
    return chapterList.slice(first, last + 1).map(chapter => chapter.id)
  }, [chapterList, extendedChapter, position.readId, visibleChapters])
  const manifests = useQueries({
    queries: windowIds.map(manifestOptions),
    combine: results =>
      results.map((result, index) => ({
        id: windowIds[index],
        data: result.data,
        error: result.error,
        loading: result.isLoading,
        fetching: result.isFetching
      }))
  })
  const activeManifest = manifests.find(item => item.id === position.readId)
  const pageCount = activeManifest?.data?.pages.length ?? 0
  const selectedIndex = pageCount ? Math.min(position.index, pageCount - 1) : position.index
  const pages = useMemo(() => {
    const active = manifests.findIndex(item => item.id === position.readId)
    if (!manifests[active]?.data?.pages.length) return []
    let first = active
    let last = active
    while (first > 0 && manifests[first - 1].data?.pages.length) first--
    while (last + 1 < manifests.length && manifests[last + 1].data?.pages.length) last++
    return manifests.slice(first, last + 1).flatMap(item =>
      item.data!.pages.map((page, localIndex): ReaderPageItem => {
        const key = item.id + ':' + localIndex
        return { ...page, ...imageSizes.get(key), key, chapterId: item.id, localIndex }
      })
    )
  }, [imageSizes, manifests, position.readId])
  const selectedGlobal = pages.findIndex(
    page => page.chapterId === position.readId && page.localIndex === selectedIndex
  )
  const spread = readerSpread(pages, Math.max(0, selectedGlobal), pageStep > 1)
  const globalIndex = pages.length ? spread.start : selectedIndex
  const currentPage = pages[globalIndex]
  const currentIndex = currentPage?.localIndex ?? selectedIndex
  const activeReadId = currentPage?.chapterId ?? position.readId
  const visiblePageCount = spread.end - spread.start
  const pageWindow = useMemo(
    (): ReaderWindowPage[] =>
      pages
        .slice(Math.max(0, globalIndex - 2), Math.min(pages.length, spread.end + 2))
        .map(page => ({
          index: pages.indexOf(page),
          src: page.path,
          key: page.key,
          chapterId: page.chapterId,
          localIndex: page.localIndex,
          width: page.width,
          height: page.height
        })),
    [globalIndex, pages, spread.end]
  )
  const activeChapterIndex = chapterList.findIndex(chapter => chapter.id === activeReadId)
  const previousChapter = chapterList[activeChapterIndex - 1] ?? null
  const nextChapter = chapterList[activeChapterIndex + 1] ?? null

  useEffect(() => {
    if (pageCount && position.index >= pageCount) {
      setPosition(current => ({ ...current, index: pageCount - 1 }))
    }
  }, [pageCount, position.index])

  useEffect(
    () => () => {
      navigationEpoch.current++
    },
    []
  )

  useEffect(() => {
    if (!paged) setTransitionDirection(null)
  }, [paged])

  const dismissTransition = useCallback(() => {
    navigationEpoch.current++
    jumpRef.current = null
    setJump(null)
    setTransitionDirection(null)
  }, [])

  const commitNavigation = useCallback((readId: string, index: number) => {
    setTransitionDirection(null)
    setPosition({ readId, index })
    setVisibleChapters([])
    setExtendedChapter(null)
    setNavigationCommand(current => ({
      id: current.id + 1,
      targetIndex: index,
      targetKey: readId + ':' + index
    }))
  }, [])

  const goToChapter = useCallback(
    async (readId: string, index = 0, keepTransition = false) => {
      if (
        jumpRef.current?.loading &&
        jumpRef.current.readId === readId &&
        jumpRef.current.index === index
      )
        return
      const epoch = ++navigationEpoch.current
      if (!keepTransition) setTransitionDirection(null)
      const pending = { readId, index, loading: true, error: null }
      jumpRef.current = pending
      setJump(pending)
      try {
        const manifest = await queryClient.fetchQuery(manifestOptions(readId))
        if (!manifest.pages.length) throw new Error('该章节暂无图片')
        if (navigationEpoch.current !== epoch) return
        commitNavigation(
          readId,
          index < 0
            ? manifest.pages.length - 1
            : Math.min(normalizeIndex(index), manifest.pages.length - 1)
        )
        jumpRef.current = null
        setJump(null)
      } catch (error) {
        if (navigationEpoch.current !== epoch) return
        const failed = {
          readId,
          index,
          loading: false,
          error: error instanceof Error ? error.message : '章节加载失败'
        }
        jumpRef.current = failed
        setJump(failed)
      }
    },
    [commitNavigation, queryClient]
  )

  const goToPage = useCallback(
    (index: number) => {
      if (!pageCount) return
      navigationEpoch.current++
      jumpRef.current = null
      setJump(null)
      commitNavigation(activeReadId, Math.min(normalizeIndex(index), pageCount - 1))
    },
    [activeReadId, commitNavigation, pageCount]
  )

  const goToNextPage = useCallback(() => {
    if (!pageCount) return
    if (paged && transitionDirection === -1) {
      dismissTransition()
      return
    }
    if (jumpRef.current?.loading) return
    if (paged && transitionDirection === 1) {
      if (nextChapter) void goToChapter(nextChapter.id, 0, true)
      return
    }
    const nextPage = pages[spread.end]
    if (paged && (!nextPage || nextPage.chapterId !== activeReadId)) {
      jumpRef.current = null
      setJump(null)
      setTransitionDirection(1)
      return
    }
    if (nextPage) {
      navigationEpoch.current++
      setJump(null)
      jumpRef.current = null
      commitNavigation(nextPage.chapterId, nextPage.localIndex)
    } else if (nextChapter) {
      void goToChapter(nextChapter.id)
    }
  }, [
    activeReadId,
    commitNavigation,
    dismissTransition,
    goToChapter,
    nextChapter,
    pageCount,
    paged,
    pages,
    spread.end,
    transitionDirection
  ])

  const goToPreviousPage = useCallback(() => {
    if (!pageCount) return
    if (paged && transitionDirection === 1) {
      dismissTransition()
      return
    }
    if (jumpRef.current?.loading) return
    if (paged && transitionDirection === -1) {
      if (previousChapter) void goToChapter(previousChapter.id, -1, true)
      return
    }
    const previous = pages[spread.previous]
    if (paged && (!previous || previous.chapterId !== activeReadId)) {
      jumpRef.current = null
      setJump(null)
      setTransitionDirection(-1)
      return
    }
    if (previous) {
      navigationEpoch.current++
      setJump(null)
      jumpRef.current = null
      commitNavigation(previous.chapterId, previous.localIndex)
    } else if (previousChapter) {
      void goToChapter(previousChapter.id, -1)
    }
  }, [
    activeReadId,
    commitNavigation,
    dismissTransition,
    goToChapter,
    pageCount,
    paged,
    pages,
    previousChapter,
    spread.previous,
    transitionDirection
  ])

  const observePage = useCallback(
    (index: number) => {
      const page = pages[index]
      if (!page || jumpRef.current?.loading) return
      setTransitionDirection(null)
      setPosition(current =>
        current.readId === page.chapterId && current.index === page.localIndex
          ? current
          : { readId: page.chapterId, index: page.localIndex }
      )
      setExtendedChapter(null)
    },
    [pages]
  )

  const observeVisibleRange = useCallback(
    (first: number, last: number) => {
      const ids = [...new Set(pages.slice(first, last + 1).map(page => page.chapterId))]
      setVisibleChapters(current =>
        current.length === ids.length && current.every((id, index) => id === ids[index])
          ? current
          : ids
      )
    },
    [pages]
  )

  const onPageLoaded = useCallback(
    (page: ReaderPageItem | ReaderWindowPage, image: HTMLImageElement) => {
      const width = image.naturalWidth
      const height = image.naturalHeight
      if (width <= 0 || height <= 0) return
      setImageSizes(current => {
        const known = current.get(page.key)
        if (known?.width === width && known.height === height) return current
        const next = new Map(current)
        next.delete(page.key)
        next.set(page.key, { width, height })
        if (next.size > 1000) next.delete(next.keys().next().value!)
        return next
      })
    },
    []
  )

  const onPageReady = useCallback((key: string, ready: boolean) => {
    setReadyPages(current => {
      if (current.has(key) === ready) return current
      const next = new Set(current)
      if (ready) next.add(key)
      else next.delete(key)
      return next
    })
  }, [])

  function boundary(direction: -1 | 1): ReaderBoundary {
    const edge = direction < 0 ? pages[0] : pages[pages.length - 1]
    const edgeIndex = chapterList.findIndex(
      chapter => chapter.id === (edge?.chapterId ?? activeReadId)
    )
    const chapter = chapterList[edgeIndex + direction] ?? null
    const result = manifests.find(item => item.id === chapter?.id)
    return {
      chapter,
      loading: result?.loading ?? false,
      error:
        result?.error?.message ??
        (result?.data && !result.data.pages.length ? '该章节暂无图片' : null)
    }
  }
  const previousBoundary = boundary(-1)
  const nextBoundary = boundary(1)
  const loadBoundary = useCallback(
    (direction: -1 | 1) => {
      const target = direction < 0 ? previousBoundary.chapter : nextBoundary.chapter
      if (!target) return
      setExtendedChapter(target.id)
      void queryClient.fetchQuery(manifestOptions(target.id)).catch(() => {
        // The boundary observes this query and presents its retry action.
      })
    },
    [nextBoundary.chapter, previousBoundary.chapter, queryClient]
  )

  // A route replacement is an acknowledgement of our visible page, not a jump.
  // External route changes still enter the same guarded navigation path.
  const routeKey = comicId + ':' + normalizeIndex(initialIndex)
  const activeKey = activeReadId + ':' + currentIndex
  const lastRoute = useRef(routeKey)
  const publishedLocations = useRef(new Map<string, number>())
  const publicationSequence = useRef(0)
  const lastPublished = useRef<string | null>(null)
  useEffect(() => {
    if (lastRoute.current !== routeKey) {
      lastRoute.current = routeKey
      if (publishedLocations.current.has(routeKey)) {
        const acknowledged = publishedLocations.current.get(routeKey)!
        for (const [key, sequence] of publishedLocations.current) {
          if (sequence <= acknowledged) publishedLocations.current.delete(key)
        }
      } else {
        lastPublished.current = null
        void goToChapter(comicId, initialIndex)
        return
      }
    }
    if (
      !onLocationChange ||
      !pageCount ||
      jump?.loading ||
      activeKey === routeKey ||
      lastPublished.current === activeKey
    )
      return
    lastPublished.current = activeKey
    publishedLocations.current.set(activeKey, ++publicationSequence.current)
    void Promise.resolve(onLocationChange(activeReadId, currentIndex)).catch(() => {
      publishedLocations.current.delete(activeKey)
      lastPublished.current = null
    })
  }, [
    activeKey,
    activeReadId,
    comicId,
    currentIndex,
    goToChapter,
    initialIndex,
    jump?.loading,
    onLocationChange,
    pageCount,
    routeKey
  ])

  const transitionChapter = transitionDirection === -1 ? previousChapter : nextChapter
  const transitionManifest = manifests.find(item => item.id === transitionChapter?.id)
  const transition =
    paged && transitionDirection != null
      ? {
          direction: transitionDirection,
          currentChapter: chapterList[activeChapterIndex],
          boundary: {
            chapter: transitionChapter,
            loading: Boolean(jump?.loading || transitionManifest?.loading),
            error:
              jump?.error ??
              transitionManifest?.error?.message ??
              (transitionManifest?.data && !transitionManifest.data.pages.length
                ? '该章节暂无图片'
                : null)
          }
        }
      : null

  return {
    activeReadId,
    currentIndex,
    globalIndex,
    pageCount,
    pages,
    pageSrc: currentPage?.path ?? '',
    pageWindow,
    visiblePageCount,
    navigationCommand,
    previousChapter,
    nextChapter,
    previousBoundary,
    nextBoundary,
    jump,
    transition,
    isLastPage: pageCount > 0 && currentIndex + visiblePageCount >= pageCount,
    isManifestLoading: !pageCount && Boolean(activeManifest?.loading),
    manifestError:
      activeManifest?.error ??
      (activeManifest?.data && !pageCount ? new Error('当前章节暂无图片') : null),
    isFetching: Boolean(activeManifest?.fetching || jump?.loading),
    isPageReady:
      pages.length > 0 &&
      pages.slice(spread.start, spread.end).every(page => readyPages.has(page.key)),
    goToPreviousPage,
    goToNextPage,
    goToPage,
    goToChapter,
    observePage,
    observeVisibleRange,
    onPageLoaded,
    onPageReady,
    loadBoundary,
    retry: () => {
      if (transition?.boundary.chapter) {
        return goToChapter(transition.boundary.chapter.id, transition.direction < 0 ? -1 : 0, true)
      } else if (jump) return goToChapter(jump.readId, jump.index)
      return queryClient.invalidateQueries({ queryKey: queryKeys.readerManifest(activeReadId) })
    }
  }
}

function normalizeIndex(index: number) {
  return Number.isFinite(index) ? Math.max(0, Math.floor(index)) : 0
}
