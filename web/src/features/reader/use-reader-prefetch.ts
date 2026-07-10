import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useRef } from 'react'

import { CACHE, READER } from '@/lib/constants'
import type { ReaderPageQueryKeyFactory, ReaderPageRequester } from './use-reader-page-query'

export function useReaderPrefetch({
  comicId,
  endpoint,
  cacheLimitBytes,
  currentIndex,
  pageCount,
  pageStep,
  enabled,
  pageQueryKey,
  requestPage
}: {
  comicId: string
  endpoint: string
  cacheLimitBytes: number
  currentIndex: number
  pageCount: number
  pageStep: number
  enabled: boolean
  pageQueryKey: ReaderPageQueryKeyFactory
  requestPage: ReaderPageRequester
}) {
  const queryClient = useQueryClient()
  const prefetchKeyRef = useRef('')

  useEffect(() => {
    if (!enabled) {
      return
    }

    const prefetchIndexes = readerPrefetchIndexes(
      currentIndex,
      pageCount,
      pageStep,
      READER.PREFETCH_RADIUS
    )

    if (prefetchIndexes.length === 0) {
      return
    }

    const prefetchKey = [
      endpoint,
      cacheLimitBytes,
      comicId,
      currentIndex,
      prefetchIndexes.join(',')
    ].join('|')

    if (prefetchKeyRef.current === prefetchKey) {
      return
    }

    prefetchKeyRef.current = prefetchKey
    let isActive = true

    void Promise.allSettled(
      prefetchIndexes.map(index =>
        queryClient.prefetchQuery({
          queryKey: pageQueryKey(index),
          queryFn: () => requestPage(index, 'prefetch'),
          staleTime: CACHE.READER_STALE_TIME,
          gcTime: CACHE.READER_GC_TIME,
          retry: false
        })
      )
    ).then(results => {
      if (!isActive || !import.meta.env.DEV) {
        return
      }

      for (const result of results) {
        if (result.status === 'rejected') {
          console.debug('Reader page prefetch failed', result.reason)
        }
      }
    })

    return () => {
      isActive = false
    }
  }, [
    cacheLimitBytes,
    comicId,
    currentIndex,
    enabled,
    endpoint,
    pageCount,
    pageStep,
    pageQueryKey,
    queryClient,
    requestPage
  ])
}

function readerPrefetchIndexes(
  currentIndex: number,
  pageCount: number,
  pageStep: number,
  radius: number
) {
  const normalizedPageStep = Math.max(1, Math.floor(pageStep))
  const indexes: number[] = []

  for (let groupDistance = 1; groupDistance <= radius; groupDistance += 1) {
    const nextStart = currentIndex + groupDistance * normalizedPageStep
    const previousStart = currentIndex - groupDistance * normalizedPageStep

    for (let offset = 0; offset < normalizedPageStep; offset += 1) {
      const nextIndex = nextStart + offset

      if (nextIndex < pageCount) {
        indexes.push(nextIndex)
      }
    }

    for (let offset = 0; offset < normalizedPageStep; offset += 1) {
      const previousIndex = previousStart + offset

      if (previousIndex >= 0 && previousIndex < pageCount) {
        indexes.push(previousIndex)
      }
    }
  }

  return [...new Set(indexes)]
}
