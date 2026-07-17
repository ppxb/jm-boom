import type { ComicSummary } from '@/domain/comic'
import { apiClient } from './client'

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

export function listReadingHistory(): Promise<ReadingHistoryListResult> {
  return apiClient.get('/api/history')
}

export function upsertReadingHistory(
  item: ReadingHistoryItem,
  keepalive = false
): Promise<void> {
  return apiClient.put<void>(
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

export function removeReadingHistory(comicIds: string[]): Promise<void> {
  return apiClient.post<void>('/api/history/remove', { comicIds })
}

export function clearReadingHistory(): Promise<void> {
  return apiClient.delete('/api/history')
}
