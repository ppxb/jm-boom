import { useCallback, useEffect, useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'

import type { ComicDetail } from '@/domain/comic'
import { getComicReadManifest } from '@/lib/api/reader'
import { CACHE, READER } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { clearReaderPreloadScope, setReaderPreloadScope } from '@/lib/reader-preload'
import { useSettingsStore } from '@/stores/settings-store'
import type { ComicReadingTarget } from './reading-target'

export function useComicReaderPreload(comic: ComicDetail, readingTarget: ComicReadingTarget) {
  const queryClient = useQueryClient()
  const hideCovers = useSettingsStore(state => state.hideCovers)
  const [settledCoverUrl, setSettledCoverUrl] = useState('')
  const preloadScope = `detail:${comic.id}`
  const isCoverSettled = hideCovers || comic.image.length === 0 || settledCoverUrl === comic.image

  useEffect(() => {
    if (!isCoverSettled) {
      return
    }

    const readId = readingTarget.readId.trim()

    if (readId.length === 0) {
      return
    }

    let isActive = true

    void queryClient
      .fetchQuery({
        queryKey: queryKeys.readerManifest(readId),
        queryFn: () => getComicReadManifest({ readId }),
        staleTime: CACHE.READER_STALE_TIME,
        gcTime: CACHE.READER_GC_TIME
      })
      .then(manifest => {
        if (!isActive) {
          return
        }

        const initialPageIndex = Math.max((readingTarget.page ?? 1) - 1, 0)
        const startIndex = Math.min(initialPageIndex, Math.max(manifest.pages.length - 1, 0))
        const paths = manifest.pages
          .slice(startIndex, startIndex + READER.PREFETCH_AHEAD_PAGES)
          .map(page => page.path)
        setReaderPreloadScope(preloadScope, paths)
      })
      .catch(error => {
        if (isActive && import.meta.env.DEV) {
          console.debug('Comic detail reader manifest prefetch failed', error)
        }
      })

    return () => {
      isActive = false
      clearReaderPreloadScope(preloadScope)
    }
  }, [isCoverSettled, preloadScope, queryClient, readingTarget.page, readingTarget.readId])

  return useCallback(() => setSettledCoverUrl(comic.image), [comic.image])
}
