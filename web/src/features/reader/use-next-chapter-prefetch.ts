import { useQueryClient } from '@tanstack/react-query'
import { useEffect, useRef } from 'react'

import { getComicReadManifest, getComicReadPage } from '@/lib/api/reader'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import type { ReaderChapterItem } from './types'

const NEXT_CHAPTER_PREFETCH_REMAINING_PAGES = 6
const NEXT_CHAPTER_PREFETCH_PROGRESS = 0.8
const NEXT_CHAPTER_PREFETCH_INITIAL_PAGES = 2

export function useNextChapterPrefetch({
  currentIndex,
  pageCount,
  nextChapter,
  pageStep
}: {
  currentIndex: number
  pageCount: number
  nextChapter: ReaderChapterItem | null
  pageStep: number
}) {
  const queryClient = useQueryClient()
  const prefetchedChapterRef = useRef('')

  useEffect(() => {
    const nextReadId = nextChapter?.id ?? ''

    if (!nextReadId || pageCount <= 0 || !shouldPrefetchNextChapter(currentIndex, pageCount)) {
      return
    }

    const prefetchKey = [nextReadId, pageStep].join('|')

    if (prefetchedChapterRef.current === prefetchKey) {
      return
    }

    prefetchedChapterRef.current = prefetchKey
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

        const manifest = queryClient.getQueryData<Awaited<ReturnType<typeof getComicReadManifest>>>(
          queryKeys.readerManifest(nextReadId)
        )
        const initialPageCount = Math.min(
          manifest?.pageCount ?? 0,
          Math.max(NEXT_CHAPTER_PREFETCH_INITIAL_PAGES, pageStep)
        )

        if (initialPageCount <= 0) {
          return
        }

        return Promise.allSettled(
          Array.from({ length: initialPageCount }, (_, index) =>
            queryClient.prefetchQuery({
              queryKey: queryKeys.readerPage(nextReadId, index),
              queryFn: () =>
                getComicReadPage({
                  readId: nextReadId,
                  index,
                  requestOrigin: 'prefetch'
                }),
              staleTime: CACHE.READER_STALE_TIME,
              gcTime: CACHE.READER_GC_TIME,
              retry: false
            })
          )
        )
      })
      .catch(error => {
        if (import.meta.env.DEV) {
          console.debug('Reader next chapter prefetch failed', error)
        }
      })

    return () => {
      isActive = false
    }
  }, [currentIndex, nextChapter, pageCount, pageStep, queryClient])
}

function shouldPrefetchNextChapter(currentIndex: number, pageCount: number) {
  const remainingPages = pageCount - currentIndex - 1
  const progress = pageCount > 0 ? (currentIndex + 1) / pageCount : 0

  return (
    remainingPages <= NEXT_CHAPTER_PREFETCH_REMAINING_PAGES ||
    progress >= NEXT_CHAPTER_PREFETCH_PROGRESS
  )
}
