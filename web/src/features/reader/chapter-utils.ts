import type { ComicChapter } from '@/lib/api/comic'
import { formatComicChapterTitle, sortComicChapters } from '@/lib/comic'
import type { ReaderChapterItem } from './types'

export function resolveReaderChapterInfo({
  currentReadId,
  chapters,
  fallback
}: {
  currentReadId: string
  chapters: ComicChapter[]
  fallback: string
}) {
  const chapterItems = toReaderChapterItems(chapters)
  const currentIndex = chapterItems.findIndex(chapter => chapter.id === currentReadId)
  const currentChapter = currentIndex >= 0 ? chapterItems[currentIndex] : null

  return {
    chapterTitle: currentChapter?.title ?? fallback.trim(),
    chapters: chapterItems,
    currentChapter,
    previousChapter: currentIndex >= 0 ? (chapterItems[currentIndex + 1] ?? null) : null,
    nextChapter: currentIndex >= 0 ? (chapterItems[currentIndex - 1] ?? null) : null
  }
}

export function toReaderChapterItems(chapters: ComicChapter[]): ReaderChapterItem[] {
  return sortComicChapters(chapters).map((chapter, index) => ({
    id: chapter.id,
    title: formatComicChapterTitle(chapter, index)
  }))
}
