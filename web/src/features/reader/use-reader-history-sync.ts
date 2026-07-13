import { useEffect, useMemo, useRef } from 'react'

import { READER } from '@/lib/constants'
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

  pendingHistoryRef.current = historyItem

  useEffect(() => {
    if (!historyItem) {
      return
    }

    const timeout = window.setTimeout(() => upsertReadingHistory(historyItem), 400)

    return () => window.clearTimeout(timeout)
  }, [historyItem, upsertReadingHistory])

  useEffect(() => {
    const flushPendingHistory = () => {
      const pendingHistory = pendingHistoryRef.current

      if (pendingHistory) {
        upsertReadingHistory(pendingHistory)
      }
    }

    window.addEventListener('pagehide', flushPendingHistory)

    return () => {
      window.removeEventListener('pagehide', flushPendingHistory)
      flushPendingHistory()
    }
  }, [upsertReadingHistory])
}
