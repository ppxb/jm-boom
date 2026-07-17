import { useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef } from 'react'

import { HISTORY, READER } from '@/lib/constants'
import {
  upsertReadingHistory,
  type ReadingHistoryItem,
  type ReadingHistoryListResult
} from '@/lib/api/history'
import { queryKeys } from '@/lib/query-keys'

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
  const queryClient = useQueryClient()
  const pendingHistoryRef = useRef<Omit<ReadingHistoryItem, 'lastReadAt'> | null>(null)
  const lastPersistedAtRef = useRef(0)
  const historyItem = useMemo(
    () =>
      comicId && pageCount > 0
        ? {
            id: albumId || comicId,
            title: title || `JM ${albumId || comicId}`,
            author,
            image: coverUrl,
            chapterId: comicId,
            chapterTitle: chapter || READER.DEFAULT_CHAPTER_TITLE,
            pageIndex: currentIndex,
            pageCount
          }
        : null,
    [albumId, author, chapter, comicId, coverUrl, currentIndex, pageCount, title]
  )

  const flushPendingHistory = useCallback(
    (keepalive = false) => {
      const pendingHistory = pendingHistoryRef.current

      if (!pendingHistory) {
        return
      }

      pendingHistoryRef.current = null
      const lastReadAt = Date.now()
      lastPersistedAtRef.current = lastReadAt
      const nextItem = { ...pendingHistory, lastReadAt }
      queryClient.setQueryData<ReadingHistoryListResult>(queryKeys.readingHistory(), current => ({
        items: mergeHistoryItem(current?.items ?? [], nextItem)
      }))
      void upsertReadingHistory(nextItem, keepalive)
        .then(result =>
          queryClient.setQueryData<ReadingHistoryListResult>(
            queryKeys.readingHistory(),
            current => ({
              items: mergeHistoryLists(current?.items ?? [], result.items)
            })
          )
        )
        .catch(error => {
          if (import.meta.env.DEV) {
            console.debug('Reading history sync failed', error)
          }
        })
    },
    [queryClient]
  )

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
    const handlePageHide = () => flushPendingHistory(true)
    window.addEventListener('pagehide', handlePageHide)

    return () => {
      window.removeEventListener('pagehide', handlePageHide)
      flushPendingHistory()
    }
  }, [flushPendingHistory])
}

function mergeHistoryItem(items: ReadingHistoryItem[], nextItem: ReadingHistoryItem) {
  return [nextItem, ...items.filter(item => item.id !== nextItem.id)]
}

function mergeHistoryLists(current: ReadingHistoryItem[], incoming: ReadingHistoryItem[]) {
  const itemsById = new Map(current.map(item => [item.id, item]))

  for (const item of incoming) {
    const existing = itemsById.get(item.id)
    if (!existing || item.lastReadAt >= existing.lastReadAt) {
      itemsById.set(item.id, item)
    }
  }

  return [...itemsById.values()].sort((left, right) => right.lastReadAt - left.lastReadAt)
}
