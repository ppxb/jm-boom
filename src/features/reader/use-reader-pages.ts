import { useQueries, useQuery } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import {
  type ComicReadPageResult,
  getComicReadManifest,
  getComicReadPage,
  prefetchComicReadPages,
  readerFileSrc
} from '@/lib/api/reader'
import {
  PREFETCH_SETTLE_MS,
  READER_GC_TIME,
  READER_STALE_TIME
} from './constants'
import { useSettingsStore } from '@/stores/settings-store'

const WARMED_READER_IMAGES = new Set<string>()

type PrefetchWindow = {
  key: string
  centerIndex: number
}

export function useReaderPages(comicId: string, initialIndex = 0) {
  const endpoint = useSettingsStore(state => state.api)
  const shunt = useSettingsStore(state => state.shunt)
  const prefetchCount = useSettingsStore(state => state.prefetchCount)
  const readerCacheLimitMb = useSettingsStore(state => state.readerCacheLimitMb)
  const cacheLimitBytes = readerCacheLimitMb * 1024 * 1024
  const initialPageIndex = normalizePageIndex(initialIndex)
  const [currentIndex, setCurrentIndex] = useState(initialPageIndex)
  const lastPrefetchWindowRef = useRef<PrefetchWindow | null>(null)

  const manifest = useQuery({
    queryKey: ['jm-reader-manifest', endpoint, shunt, comicId],
    queryFn: () => getComicReadManifest({ readId: comicId, shunt, endpoint }),
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const pageCount = manifest.data?.pageCount ?? 0
  const clampPageIndex = useCallback(
    (index: number) => Math.min(Math.max(index, 0), Math.max(pageCount - 1, 0)),
    [pageCount]
  )
  const effectiveCurrentIndex = pageCount > 0 ? clampPageIndex(currentIndex) : currentIndex
  const pageQueryKey = useCallback(
    (index: number) =>
      [
        'jm-reader-page',
        endpoint,
        shunt,
        cacheLimitBytes,
        comicId,
        index,
        manifest.data?.shunt
      ] as const,
    [cacheLimitBytes, comicId, endpoint, manifest.data?.shunt, shunt]
  )
  const fetchPage = useCallback(
    (index: number) =>
      getComicReadPage({
        readId: comicId,
        index,
        shunt: manifest.data?.shunt ?? shunt,
        endpoint,
        cacheLimitBytes
      }),
    [cacheLimitBytes, comicId, endpoint, manifest.data?.shunt, shunt]
  )
  const page = useQuery({
    queryKey: pageQueryKey(effectiveCurrentIndex),
    queryFn: () => fetchPage(effectiveCurrentIndex),
    enabled: manifest.isSuccess && pageCount > 0,
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const isPageReady = page.data?.index === effectiveCurrentIndex
  const warmPageIndexes = useMemo(() => {
    if (pageCount === 0) {
      return []
    }

    return [effectiveCurrentIndex + 1, effectiveCurrentIndex - 1].filter(
      index => index >= 0 && index < pageCount
    )
  }, [effectiveCurrentIndex, pageCount, prefetchCount])
  const warmPages = useQueries({
    queries: warmPageIndexes.map(index => ({
      queryKey: pageQueryKey(index),
      queryFn: () => fetchPage(index),
      enabled: manifest.isSuccess && isPageReady,
      staleTime: READER_STALE_TIME,
      gcTime: READER_GC_TIME,
      refetchOnMount: false,
      refetchOnWindowFocus: false
    }))
  })

  const goToPreviousPage = useCallback(() => {
    if (pageCount === 0) {
      return
    }

    setCurrentIndex(index => clampPageIndex(index - 1))
  }, [clampPageIndex, pageCount])
  const goToNextPage = useCallback(() => {
    if (pageCount === 0) {
      return
    }

    setCurrentIndex(index => clampPageIndex(index + 1))
  }, [clampPageIndex, pageCount])
  const retry = useCallback(() => {
    if (manifest.isError) {
      void manifest.refetch()
      return
    }

    void page.refetch()
  }, [manifest, page])
  const pageSrc = useMemo(
    () => (isPageReady && page.data ? readerFileSrc(page.data.path) : ''),
    [isPageReady, page.data]
  )
  const isLastPage = pageCount > 0 && currentIndex >= pageCount - 1

  useEffect(() => {
    setCurrentIndex(initialPageIndex)
  }, [comicId, endpoint, initialPageIndex, shunt])

  useEffect(() => {
    if (currentIndex < pageCount || pageCount === 0) {
      return
    }

    setCurrentIndex(Math.max(0, pageCount - 1))
  }, [currentIndex, pageCount])

  useEffect(() => {
    warmPages.forEach(result => {
      const page = result.data as ComicReadPageResult | undefined

      if (page) {
        warmReaderImage(readerFileSrc(page.path))
      }
    })
  }, [warmPages])

  useEffect(() => {
    if (!manifest.data || pageCount === 0 || !isPageReady) {
      return
    }

    const prefetchStep = Math.max(1, Math.ceil(prefetchCount / 2))
    const prefetchKey = [
      endpoint,
      manifest.data.shunt,
      cacheLimitBytes,
      comicId,
      pageCount,
      prefetchCount
    ].join('|')
    const lastPrefetchWindow = lastPrefetchWindowRef.current

    if (
      lastPrefetchWindow?.key === prefetchKey &&
      Math.abs(effectiveCurrentIndex - lastPrefetchWindow.centerIndex) < prefetchStep
    ) {
      return
    }

    const timer = window.setTimeout(() => {
      lastPrefetchWindowRef.current = {
        key: prefetchKey,
        centerIndex: effectiveCurrentIndex
      }

      void prefetchComicReadPages({
        readId: comicId,
        centerIndex: effectiveCurrentIndex,
        radius: prefetchCount,
        shunt: manifest.data.shunt,
        endpoint,
        cacheLimitBytes
      }).catch(error => {
        console.debug('Reader prefetch failed', error)
      })
    }, PREFETCH_SETTLE_MS)

    return () => window.clearTimeout(timer)
  }, [
    cacheLimitBytes,
    comicId,
    effectiveCurrentIndex,
    endpoint,
    isPageReady,
    manifest.data,
    pageCount,
    prefetchCount
  ])

  return {
    currentIndex,
    pageCount,
    pageSrc,
    isLastPage,
    isManifestLoading: manifest.isLoading,
    manifestError: manifest.isError ? manifest.error : null,
    isPageLoading: page.isLoading && !page.data,
    pageError: page.isError ? page.error : null,
    isFetching: manifest.isFetching || page.isFetching,
    goToPreviousPage,
    goToNextPage,
    retry
  }
}

function warmReaderImage(src: string) {
  if (!src || WARMED_READER_IMAGES.has(src)) {
    return
  }

  WARMED_READER_IMAGES.add(src)

  const image = new Image()
  image.decoding = 'async'
  image.src = src

  void image.decode?.().catch(() => {})
}

function normalizePageIndex(index: number) {
  if (!Number.isFinite(index)) {
    return 0
  }

  return Math.max(0, Math.floor(index))
}
