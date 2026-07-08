import type { ComicChapter, ComicDetail } from '@/lib/api/comic'

export const SINGLE_CHAPTER_TITLE = '单章'

const CHINESE_DATE_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  year: 'numeric',
  month: 'long',
  day: 'numeric'
})
const CHINESE_DATE_TIME_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  year: 'numeric',
  month: 'long',
  day: 'numeric',
  hour: '2-digit',
  minute: '2-digit',
  hour12: false
})

export function sortChapters(chapters: ComicChapter[]) {
  return [...chapters].sort((left, right) => {
    const leftSort = Number.parseInt(left.sort, 10)
    const rightSort = Number.parseInt(right.sort, 10)

    if (Number.isNaN(leftSort) || Number.isNaN(rightSort)) {
      return 0
    }

    return rightSort - leftSort
  })
}

export function formatChapterTitle(chapter: ComicChapter, index: number) {
  const title = chapter.title.trim()

  if (title.length > 0) {
    return title
  }

  return chapter.sort ? `第 ${chapter.sort} 章` : `章节 ${index + 1}`
}

export function getDisplayChapterCount(chapters: ComicChapter[]) {
  return Math.max(chapters.length, 1)
}

export function getNextChapter(currentId: string, chapters: ComicChapter[]) {
  const sortedChapters = sortChapters(chapters)
  const currentIndex = sortedChapters.findIndex(chapter => chapter.id === currentId)

  if (currentIndex < 0) {
    return null
  }

  return sortedChapters[currentIndex - 1] ?? null
}

export function resolveStartReadingTarget(comic: ComicDetail) {
  const sortedChapters = sortChapters(comic.series)
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
    chapterTitle: formatChapterTitle(firstChapter, firstChapterIndex),
    nextChapter: toReaderNextChapter(sortedChapters[firstChapterIndex - 1], firstChapterIndex - 1)
  }
}

export function resolveAlbumId(comic: { id: string; seriesId?: string | null }) {
  const seriesId = comic.seriesId?.trim() ?? ''

  return seriesId.length > 0 && seriesId !== '0' ? seriesId : comic.id
}

export function getVisiblePages(currentPage: number, pageCount: number) {
  if (pageCount <= 7) {
    return Array.from({ length: pageCount }, (_, index) => index + 1)
  }

  const pages = new Set([1, pageCount, currentPage - 1, currentPage, currentPage + 1])
  const sortedPages = [...pages]
    .filter(page => page >= 1 && page <= pageCount)
    .sort((left, right) => left - right)
  const visiblePages: Array<number | 'ellipsis'> = []

  for (const page of sortedPages) {
    const previousPage = visiblePages[visiblePages.length - 1]

    if (typeof previousPage === 'number' && page - previousPage > 1) {
      visiblePages.push('ellipsis')
    }

    visiblePages.push(page)
  }

  return visiblePages
}

export function formatNumber(value: number) {
  return new Intl.NumberFormat('zh-CN', {
    notation: value >= 10000 ? 'compact' : 'standard',
    maximumFractionDigits: 1
  }).format(value)
}

export function formatCommentTime(value: string) {
  const parsed = parseCommentDate(value)

  if (parsed == null) {
    return value || '未知时间'
  }

  return parsed.hasTime
    ? CHINESE_DATE_TIME_FORMATTER.format(parsed.date)
    : CHINESE_DATE_FORMATTER.format(parsed.date)
}

export function htmlToText(value: string) {
  if (value.trim().length === 0) {
    return ''
  }

  const document = new DOMParser().parseFromString(value, 'text/html')

  return document.body.textContent?.trim() ?? value
}

function parseCommentDate(value: string) {
  const normalizedValue = value.trim()

  if (normalizedValue.length === 0) {
    return null
  }

  if (/^\d{10}$/.test(normalizedValue)) {
    return {
      date: new Date(Number(normalizedValue) * 1000),
      hasTime: true
    }
  }

  if (/^\d{13}$/.test(normalizedValue)) {
    return {
      date: new Date(Number(normalizedValue)),
      hasTime: true
    }
  }

  const localDate = parseLocalDate(normalizedValue)

  if (localDate != null) {
    return localDate
  }

  const directDate = new Date(normalizedValue)

  if (isValidDate(directDate)) {
    return {
      date: directDate,
      hasTime: hasTimeComponent(normalizedValue)
    }
  }

  return null
}

function parseLocalDate(value: string) {
  const match = /^(\d{4})[-/](\d{1,2})[-/](\d{1,2})(?:[ T](\d{1,2}):(\d{2})(?::(\d{2}))?)?$/.exec(
    value
  )

  if (match == null) {
    return null
  }

  const [, yearValue, monthValue, dayValue, hourValue, minuteValue, secondValue] = match
  const year = Number(yearValue)
  const month = Number(monthValue)
  const day = Number(dayValue)
  const hour = hourValue == null ? 0 : Number(hourValue)
  const minute = minuteValue == null ? 0 : Number(minuteValue)
  const second = secondValue == null ? 0 : Number(secondValue)
  const date = new Date(year, month - 1, day, hour, minute, second)

  if (
    date.getFullYear() !== year ||
    date.getMonth() !== month - 1 ||
    date.getDate() !== day ||
    date.getHours() !== hour ||
    date.getMinutes() !== minute ||
    date.getSeconds() !== second
  ) {
    return null
  }

  return {
    date,
    hasTime: hourValue != null
  }
}

function isValidDate(value: Date) {
  return !Number.isNaN(value.getTime())
}

function hasTimeComponent(value: string) {
  return /(?:\d{1,2}:\d{2}|T\d{2})/.test(value)
}

function toReaderNextChapter(chapter: ComicChapter | undefined, index: number) {
  if (!chapter) {
    return null
  }

  return {
    id: chapter.id,
    title: formatChapterTitle(chapter, index)
  }
}
