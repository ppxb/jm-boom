import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useRef } from 'react'

import { getComicReadManifest } from '@/lib/api/reader'
import { CACHE, READER } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { clearReaderPreloadScope, setReaderPreloadScope } from '@/lib/reader-preload'
import type { ReaderChapterItem } from './types'

export function useNextChapterPrefetch({
  currentIndex,
  pageCount,
  nextChapter,
  stripPrefetchRequested = false
}: {
  currentIndex: number
  pageCount: number
  nextChapter: ReaderChapterItem | null
  stripPrefetchRequested?: boolean
}) {
  const queryClient = useQueryClient()
  const prefetchedChapterRef = useRef('')
  const nextReadId = nextChapter?.id ?? ''
  const preloadScope = `next-chapter:${nextReadId}`
  const shouldPrefetch =
    nextReadId.length > 0 &&
    pageCount > 0 &&
    shouldPrefetchNextChapter(currentIndex, pageCount, stripPrefetchRequested)

  useEffect(() => {
    if (!shouldPrefetch) {
      return
    }

    if (prefetchedChapterRef.current === nextReadId) {
      return
    }

    prefetchedChapterRef.current = nextReadId
    let isActive = true

    void queryClient
      .prefetchQuery({
        queryKey: queryKeys.readerManifest(nextReadId),
        queryFn: () => getComicReadManifest({ readId: nextReadId }),
        staleTime: CACHE.READER_STALE_TIME,
        gcTime: CACHE.READER_GC_TIME,
        retry: false
      })
      .then(() => {
        if (!isActive) {
          return
        }

        const manifest = queryClient.getQueryData<
          Awaited<ReturnType<typeof getComicReadManifest>>
        >(queryKeys.readerManifest(nextReadId))
        setReaderPreloadScope(
          preloadScope,
          manifest?.pages
            .slice(0, READER.PREFETCH_AHEAD_PAGES)
            .map(page => page.path) ?? []
        )
      })
      .catch(error => {
        if (isActive && prefetchedChapterRef.current === nextReadId) {
          prefetchedChapterRef.current = ''
        }

        if (import.meta.env.DEV) {
          console.debug('Reader next chapter prefetch failed', error)
        }
      })

    return () => {
      isActive = false
    }
  }, [nextReadId, preloadScope, queryClient, shouldPrefetch])

  useEffect(
    () => () => {
      if (nextReadId) {
        clearReaderPreloadScope(preloadScope)
      }
    },
    [nextReadId, preloadScope]
  )
}

export function shouldPrefetchNextChapter(
  currentIndex: number,
  pageCount: number,
  stripPrefetchRequested = false
) {
  const remainingPages = pageCount - currentIndex - 1
  const progress = pageCount > 0 ? (currentIndex + 1) / pageCount : 0

  return (
    stripPrefetchRequested ||
    remainingPages <= READER.PREFETCH_AHEAD_PAGES ||
    progress >= READER.NEXT_CHAPTER_PREFETCH_PROGRESS
  )
}
