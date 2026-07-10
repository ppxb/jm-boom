import { useQueries, useQuery } from '@tanstack/react-query'
import { useCallback, useMemo } from 'react'

import { type ComicReadPageResult, getComicReadPage, readerFileSrc } from '@/lib/api/reader'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import type { ReaderWindowPage } from './types'

export type ReaderPageRequestOrigin = 'visible' | 'prefetch'
export type ReaderPageQueryKeyFactory = (index: number) => ReturnType<typeof queryKeys.readerPage>
export type ReaderPageRequester = (
  index: number,
  requestOrigin: ReaderPageRequestOrigin
) => Promise<ComicReadPageResult>

export function useReaderPageQuery({
  comicId,
  endpoint,
  cacheLimitBytes,
  pageIndex,
  enabled
}: {
  comicId: string
  endpoint: string
  cacheLimitBytes: number
  pageIndex: number
  enabled: boolean
}) {
  const pageQueryKey = useCallback<ReaderPageQueryKeyFactory>(
    (index: number) => queryKeys.readerPage(endpoint, comicId, cacheLimitBytes, index),
    [cacheLimitBytes, comicId, endpoint]
  )
  const requestPage = useCallback<ReaderPageRequester>(
    (index: number, requestOrigin: ReaderPageRequestOrigin) =>
      getComicReadPage({
        readId: comicId,
        index,
        endpoint,
        requestOrigin,
        cacheLimitBytes
      }),
    [cacheLimitBytes, comicId, endpoint]
  )
  const page = useQuery({
    queryKey: pageQueryKey(pageIndex),
    queryFn: () => requestPage(pageIndex, 'visible'),
    enabled,
    staleTime: CACHE.READER_STALE_TIME,
    gcTime: CACHE.READER_GC_TIME,
    retry: false,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const isPageReady = page.data?.index === pageIndex
  const pageSrc = useMemo(
    () => (isPageReady && page.data ? readerFileSrc(page.data.path) : ''),
    [isPageReady, page.data]
  )

  return {
    page,
    pageSrc,
    isPageReady,
    pageQueryKey,
    requestPage
  }
}

export function useAdjacentReaderPageQueries({
  pageIndex,
  pageCount,
  pageStep,
  enabled,
  pageQueryKey,
  requestPage
}: {
  pageIndex: number
  pageCount: number
  pageStep: number
  enabled: boolean
  pageQueryKey: ReaderPageQueryKeyFactory
  requestPage: ReaderPageRequester
}) {
  const adjacentIndexes = useMemo(
    () => readerWindowIndexes(pageIndex, pageCount, pageStep).filter(index => index !== pageIndex),
    [pageCount, pageIndex, pageStep]
  )
  const eagerIndexSet = useMemo(
    () => new Set(readerEagerWindowIndexes(pageIndex, pageCount, pageStep)),
    [pageCount, pageIndex, pageStep]
  )
  const adjacentQueries = useQueries({
    queries: adjacentIndexes.map(index => ({
      queryKey: pageQueryKey(index),
      queryFn: () => requestPage(index, 'prefetch'),
      enabled: enabled && eagerIndexSet.has(index),
      staleTime: CACHE.READER_STALE_TIME,
      gcTime: CACHE.READER_GC_TIME,
      retry: false,
      refetchOnMount: false,
      refetchOnWindowFocus: false
    }))
  })

  return useMemo<ReaderWindowPage[]>(
    () =>
      adjacentIndexes.flatMap((index, queryIndex) => {
        const data = adjacentQueries[queryIndex]?.data

        if (!data || data.index !== index) {
          return []
        }

        return [{ index, src: readerFileSrc(data.path) }]
      }),
    [adjacentIndexes, adjacentQueries]
  )
}

function readerWindowIndexes(currentIndex: number, pageCount: number, pageStep: number) {
  if (pageCount <= 0) {
    return []
  }

  const normalizedPageStep = Math.max(1, Math.floor(pageStep))
  const indexes: number[] = []
  const start = Math.max(0, currentIndex - normalizedPageStep)
  const end = Math.min(pageCount - 1, currentIndex + normalizedPageStep * 2 - 1)

  for (let index = start; index <= end; index += 1) {
    indexes.push(index)
  }

  return indexes
}

function readerEagerWindowIndexes(currentIndex: number, pageCount: number, pageStep: number) {
  if (pageCount <= 0) {
    return []
  }

  const normalizedPageStep = Math.max(1, Math.floor(pageStep))

  if (normalizedPageStep === 1) {
    return [
      currentIndex - 1,
      currentIndex + 1
    ].filter(index => index >= 0 && index < pageCount)
  }

  const indexes: number[] = []
  const end = Math.min(pageCount - 1, currentIndex + normalizedPageStep - 1)

  for (let index = currentIndex + 1; index <= end; index += 1) {
    indexes.push(index)
  }

  return indexes
}
