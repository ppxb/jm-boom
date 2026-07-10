import type { ComicChapter, ComicDetail } from '@/lib/api/comic'

export const SINGLE_CHAPTER_TITLE = '单章'

export function sortComicChapters(chapters: ComicChapter[]) {
  return [...chapters].sort((left, right) => {
    const leftSort = Number.parseInt(left.sort, 10)
    const rightSort = Number.parseInt(right.sort, 10)

    if (Number.isNaN(leftSort) || Number.isNaN(rightSort)) {
      return 0
    }

    return rightSort - leftSort
  })
}

export function formatComicChapterTitle(chapter: ComicChapter, index: number) {
  const title = chapter.title.trim()

  if (title.length > 0) {
    return title
  }

  return chapter.sort ? `第 ${chapter.sort} 章` : `章节 ${index + 1}`
}

export function getComicDisplayChapterCount(chapters: ComicChapter[]) {
  return Math.max(chapters.length, 1)
}

export function resolveComicAlbumId(comic: { id: string; seriesId?: string | null }) {
  const seriesId = comic.seriesId?.trim() ?? ''

  return seriesId.length > 0 && seriesId !== '0' ? seriesId : comic.id
}

export function resolveComicStartReadingTarget(comic: ComicDetail) {
  const sortedChapters = sortComicChapters(comic.series)
  const firstChapterIndex = sortedChapters.length - 1
  const firstChapter = sortedChapters[firstChapterIndex]

  if (!firstChapter) {
    return {
      readId: comic.id,
      chapterTitle: SINGLE_CHAPTER_TITLE,
      nextChapter: null
    }
  }

  return {
    readId: firstChapter.id,
    chapterTitle: formatComicChapterTitle(firstChapter, firstChapterIndex),
    nextChapter: toReaderNextChapter(sortedChapters[firstChapterIndex - 1], firstChapterIndex - 1)
  }
}

function toReaderNextChapter(chapter: ComicChapter | undefined, index: number) {
  if (!chapter) {
    return null
  }

  return {
    id: chapter.id,
    title: formatComicChapterTitle(chapter, index)
  }
}
