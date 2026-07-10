import type { ReaderChapterItem, ReaderSearch } from './types'

export function toReaderChapterSearch({
  title,
  albumId,
  chapter,
  chapters
}: {
  title: string
  albumId: string
  chapter: ReaderChapterItem
  chapters: ReaderChapterItem[]
}): ReaderSearch {
  const nextChapter = resolveNextChapterForSearch(chapter.id, chapters)

  return {
    title,
    chapter: chapter.title,
    albumId,
    fromDetail: '1',
    pageIndex: '0',
    nextId: nextChapter?.id ?? '',
    nextChapter: nextChapter?.title ?? ''
  }
}

function resolveNextChapterForSearch(currentReadId: string, chapters: ReaderChapterItem[]) {
  const currentIndex = chapters.findIndex(chapter => chapter.id === currentReadId)

  if (currentIndex <= 0) {
    return null
  }

  return chapters[currentIndex - 1]
}
