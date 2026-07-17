import type { ComicChapter } from '@/domain/comic'

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
