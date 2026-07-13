import { useCallback, useEffect, useMemo, useRef } from 'react'

import { HISTORY, READER } from '@/lib/constants'
import { useReadingHistoryStore } from '@/stores/reading-history-store'

interface UseReaderHistorySyncProps {
  comicId: string
  albumId: string
  title: string
  author: string
  coverUrl: string
  chapter: string
  currentIndex: number
  pageCount: number
}

export function useReaderHistorySync({
  comicId,
  albumId,
  title,
  author,
  coverUrl,
  chapter,
  currentIndex,
  pageCount
}: UseReaderHistorySyncProps) {
  const upsertReadingHistory = useReadingHistoryStore(state => state.upsert)
  const pendingHistoryRef = useRef<Parameters<typeof upsertReadingHistory>[0] | null>(null)
  const lastPersistedAtRef = useRef(0)
  const historyItem = useMemo(
    () =>
      comicId && pageCount > 0
        ? {
            comicId: albumId || comicId,
            albumId,
            title: title || `JM ${albumId || comicId}`,
            author,
            coverUrl,
            chapterId: comicId,
            chapterTitle: chapter || READER.DEFAULT_CHAPTER_TITLE,
            pageIndex: currentIndex,
            pageCount
          }
        : null,
    [albumId, author, chapter, comicId, coverUrl, currentIndex, pageCount, title]
  )

  const flushPendingHistory = useCallback(() => {
    const pendingHistory = pendingHistoryRef.current

    if (!pendingHistory) {
      return
    }

    pendingHistoryRef.current = null
    lastPersistedAtRef.current = Date.now()
    upsertReadingHistory(pendingHistory)
  }, [upsertReadingHistory])

  useEffect(() => {
    pendingHistoryRef.current = historyItem

    if (!historyItem) {
      return
    }

    const elapsed = Date.now() - lastPersistedAtRef.current
    const delay = Math.max(HISTORY.PERSIST_INTERVAL - elapsed, 0)
    const timeout = window.setTimeout(flushPendingHistory, delay)

    return () => window.clearTimeout(timeout)
  }, [flushPendingHistory, historyItem])

  useEffect(() => {
    window.addEventListener('pagehide', flushPendingHistory)

    return () => {
      window.removeEventListener('pagehide', flushPendingHistory)
      flushPendingHistory()
    }
  }, [flushPendingHistory])
}
