import { useQuery } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import {
  cacheComicReadChapter,
  getComicReadManifest,
  getComicReadPage,
  readerFileSrc
} from '@/lib/api/reader'
import { READER_GC_TIME, READER_STALE_TIME } from './constants'
import { useSettingsStore } from '@/stores/settings-store'

export function useReaderPages(comicId: string, initialIndex = 0) {
  const endpoint = useSettingsStore(state => state.api)
  const readerCacheLimitMb = useSettingsStore(state => state.readerCacheLimitMb)
  const cacheLimitBytes = readerCacheLimitMb * 1024 * 1024
  const initialPageIndex = normalizePageIndex(initialIndex)
  const [currentIndex, setCurrentIndex] = useState(initialPageIndex)
  const chapterCacheKeyRef = useRef('')

  const manifest = useQuery({
    queryKey: ['jm-reader-manifest', endpoint, comicId],
    queryFn: () => getComicReadManifest({ readId: comicId, endpoint }),
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
        comicId,
        cacheLimitBytes,
        index
      ] as const,
    [cacheLimitBytes, comicId, endpoint]
  )
  const page = useQuery({
    queryKey: pageQueryKey(effectiveCurrentIndex),
    queryFn: () =>
      getComicReadPage({
        readId: comicId,
        index: effectiveCurrentIndex,
        endpoint,
        requestOrigin: 'visible',
        cacheLimitBytes
      }),
    enabled: manifest.isSuccess && pageCount > 0,
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const isPageReady = page.data?.index === effectiveCurrentIndex
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
  }, [comicId, endpoint, initialPageIndex])

  useEffect(() => {
    if (currentIndex < pageCount || pageCount === 0) {
      return
    }

    setCurrentIndex(Math.max(0, pageCount - 1))
  }, [currentIndex, pageCount])

  useEffect(() => {
    if (!manifest.data || pageCount === 0) {
      return
    }

    const cacheKey = [
      endpoint,
      cacheLimitBytes,
      comicId,
      pageCount
    ].join('|')

    if (chapterCacheKeyRef.current === cacheKey) {
      return
    }

    chapterCacheKeyRef.current = cacheKey
    void cacheComicReadChapter({
      readId: comicId,
      endpoint,
      requestOrigin: 'chapter_cache',
      cacheLimitBytes
    }).catch(error => {
      console.debug('Reader chapter cache failed', error)
    })
  }, [cacheLimitBytes, comicId, endpoint, manifest.data, pageCount])

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

function normalizePageIndex(index: number) {
  if (!Number.isFinite(index)) {
    return 0
  }

  return Math.max(0, Math.floor(index))
}
