import type { ComicChapter, ComicDetail } from '@/domain/comic'
import type { ReadingHistoryItem } from '@/lib/api/history'

export type ComicReadingTarget = {
  readId: string
  page?: number
  isContinue: boolean
}

export function resolveComicReadingTarget(
  comic: ComicDetail,
  sortedChapters: ComicChapter[],
  history: ReadingHistoryItem | undefined
): ComicReadingTarget {
  if (history && isValidHistoryChapter(comic, history.chapterId)) {
    const maxPageIndex = Math.max(history.pageCount - 1, 0)
    const pageIndex = Math.min(normalizePageIndex(history.pageIndex), maxPageIndex)

    return {
      readId: history.chapterId,
      page: pageIndex + 1,
      isContinue: true
    }
  }

  return {
    readId: sortedChapters[sortedChapters.length - 1]?.id ?? comic.id,
    isContinue: false
  }
}

function isValidHistoryChapter(comic: ComicDetail, chapterId: string) {
  if (!chapterId) {
    return false
  }

  if (comic.chapters.length === 0) {
    return chapterId === comic.id
  }

  return comic.chapters.some(chapter => chapter.id === chapterId)
}

function normalizePageIndex(pageIndex: number) {
  if (!Number.isFinite(pageIndex)) {
    return 0
  }

  return Math.max(0, Math.floor(pageIndex))
}
