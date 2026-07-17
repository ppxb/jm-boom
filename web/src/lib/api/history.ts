import type { ComicSummary } from '@/domain/comic'
import { apiClient } from './client'

const LOCAL_HISTORY_KEY = 'jm-boom-reading-history-v2'

type ReadingHistoryComic = Pick<ComicSummary, 'id' | 'title' | 'author' | 'image'>

export type ReadingHistoryItem = ReadingHistoryComic & {
  chapterId: string
  chapterTitle: string
  pageIndex: number
  pageCount: number
  lastReadAt: number
}

export type ReadingHistoryListResult = {
  items: ReadingHistoryItem[]
}

export async function listReadingHistory(): Promise<ReadingHistoryListResult> {
  const result = await apiClient.get<ReadingHistoryListResult>('/api/history')
  const localItems = readLocalHistory()

  if (localItems.length === 0) {
    return result
  }

  const imported = await apiClient.post<ReadingHistoryListResult>('/api/history/import', {
    items: localItems
  })
  try {
    localStorage.removeItem(LOCAL_HISTORY_KEY)
  } catch {
    // Import is idempotent, so retaining the value only causes a harmless retry.
  }
  return imported
}

export function upsertReadingHistory(
  item: ReadingHistoryItem,
  keepalive = false
): Promise<ReadingHistoryListResult> {
  return apiClient.put<ReadingHistoryListResult>(
    `/api/history/${item.id}`,
    {
      title: item.title,
      author: item.author,
      image: item.image,
      chapterId: item.chapterId,
      chapterTitle: item.chapterTitle,
      pageIndex: item.pageIndex,
      pageCount: item.pageCount,
      lastReadAt: item.lastReadAt
    },
    { keepalive }
  )
}

export function removeReadingHistory(comicIds: string[]): Promise<ReadingHistoryListResult> {
  return apiClient.post<ReadingHistoryListResult>('/api/history/remove', { comicIds })
}

export function clearReadingHistory(): Promise<ReadingHistoryListResult> {
  return apiClient.delete<ReadingHistoryListResult>('/api/history')
}

function readLocalHistory(): ReadingHistoryItem[] {
  try {
    const rawValue = localStorage.getItem(LOCAL_HISTORY_KEY)
    if (!rawValue) return []
    const value: unknown = JSON.parse(rawValue)
    if (!isRecord(value) || !isRecord(value.state) || !Array.isArray(value.state.items)) {
      return []
    }
    return value.state.items.filter(isReadingHistoryItem)
  } catch {
    return []
  }
}

function isReadingHistoryItem(value: unknown): value is ReadingHistoryItem {
  return (
    isRecord(value) &&
    typeof value.id === 'string' &&
    typeof value.title === 'string' &&
    typeof value.author === 'string' &&
    typeof value.image === 'string' &&
    typeof value.chapterId === 'string' &&
    typeof value.chapterTitle === 'string' &&
    typeof value.pageIndex === 'number' &&
    Number.isInteger(value.pageIndex) &&
    value.pageIndex >= 0 &&
    typeof value.pageCount === 'number' &&
    Number.isInteger(value.pageCount) &&
    value.pageCount > 0 &&
    value.pageIndex < value.pageCount &&
    typeof value.lastReadAt === 'number' &&
    Number.isFinite(value.lastReadAt) &&
    /^\d+$/.test(value.id) &&
    /^\d+$/.test(value.chapterId)
  )
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}
