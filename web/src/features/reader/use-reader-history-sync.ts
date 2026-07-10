import { useEffect } from 'react'

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

  useEffect(() => {
    if (!comicId || pageCount <= 0) {
      return
    }

    const historyComicId = albumId || comicId
    const historyTitle = title || `JM ${historyComicId}`
    const historyChapter = chapter || READER.DEFAULT_CHAPTER_TITLE

    upsertReadingHistory({
      comicId: historyComicId,
      albumId,
      title: historyTitle,
      author,
      coverUrl,
      chapterId: comicId,
      chapterTitle: historyChapter,
      pageIndex: currentIndex,
      pageCount
    })
  }, [author, chapter, comicId, coverUrl, currentIndex, pageCount, albumId, title, upsertReadingHistory])
}
