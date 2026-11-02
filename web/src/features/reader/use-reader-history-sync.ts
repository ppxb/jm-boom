import { useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef } from 'react'

import { HISTORY, READER } from '@/lib/constants'
import { ApiError } from '@/lib/api/client'
import type { ComicStateResult } from '@/lib/api/comic'
import { upsertReadingHistory, type ReadingHistoryItem } from '@/lib/api/history'
import { queryKeys } from '@/lib/query-keys'

const MAX_RETRY_DELAY = 30_000
// Keep reading events ordered across chapter sessions, even within the same millisecond.
let lastHistoryTimestamp = 0

interface HistorySyncState {
  pending: ReadingHistoryItem | null
  request: { item: ReadingHistoryItem; keepalive: boolean } | null
  timer: number | undefined
  active: boolean
  nextAttemptAt: number
  retryDelay: number
  retryAllowed: boolean
}

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
  const syncRef = useRef<HistorySyncState>({
    pending: null,
    request: null,
    timer: undefined,
    active: false,
    nextAttemptAt: 0,
    retryDelay: HISTORY.PERSIST_INTERVAL,
    retryAllowed: true
  })
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
    async function flushPendingHistory(keepalive = false): Promise<void> {
      const state = syncRef.current
      const item = state.pending
      if (
        !item ||
        !state.retryAllowed ||
        (state.request && (!keepalive || (state.request.keepalive && state.request.item === item)))
      ) {
        return
      }

      const request = { item, keepalive }
      state.request = request
      state.nextAttemptAt = Date.now() + HISTORY.PERSIST_INTERVAL
      scheduleHistoryFlush(state, flushPendingHistory)

      try {
        await upsertReadingHistory(item, keepalive)
        if (state.pending === item) {
          state.pending = null
          state.retryDelay = HISTORY.PERSIST_INTERVAL
        }
        queryClient.setQueryData<ComicStateResult>(queryKeys.comicState(item.id), current => {
          if (!current || (current.history && current.history.lastReadAt > item.lastReadAt)) {
            return current
          }
          return { ...current, history: item }
        })
        void queryClient.invalidateQueries({ queryKey: queryKeys.readingHistory() })
      } catch (error) {
        if (state.pending === item) {
          state.retryAllowed = !(error instanceof ApiError) || error.retryable
          state.nextAttemptAt = Date.now() + state.retryDelay
          state.retryDelay = Math.min(state.retryDelay * 2, MAX_RETRY_DELAY)
        }
        if (import.meta.env.DEV) {
          console.debug('Reading history sync failed', error)
        }
      } finally {
        if (state.request === request) state.request = null
        scheduleHistoryFlush(state, flushPendingHistory)
      }
    },
    [queryClient]
  )

  useEffect(() => {
    if (!historyItem) return
    const state = syncRef.current
    lastHistoryTimestamp = Math.max(Date.now(), lastHistoryTimestamp + 1)
    state.pending = { ...historyItem, lastReadAt: lastHistoryTimestamp }
    state.retryDelay = HISTORY.PERSIST_INTERVAL
    state.retryAllowed = true
    state.nextAttemptAt = Math.min(state.nextAttemptAt, Date.now() + HISTORY.PERSIST_INTERVAL)
    scheduleHistoryFlush(state, flushPendingHistory)
  }, [flushPendingHistory, historyItem])

  useEffect(() => {
    const state = syncRef.current
    state.active = true
    scheduleHistoryFlush(state, flushPendingHistory)
    const handlePageHide = () => void flushPendingHistory(true)
    window.addEventListener('pagehide', handlePageHide)

    return () => {
      state.active = false
      window.removeEventListener('pagehide', handlePageHide)
      scheduleHistoryFlush(state, flushPendingHistory)
      void flushPendingHistory(true)
    }
  }, [flushPendingHistory])
}

function scheduleHistoryFlush(state: HistorySyncState, flush: () => Promise<void>) {
  window.clearTimeout(state.timer)
  state.timer = undefined
  if (!state.active || !state.pending || state.request || !state.retryAllowed) return
  state.timer = window.setTimeout(() => void flush(), Math.max(state.nextAttemptAt - Date.now(), 0))
}
