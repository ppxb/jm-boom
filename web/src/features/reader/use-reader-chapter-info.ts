import { useQuery } from '@tanstack/react-query'
import { useMemo } from 'react'

import { getComicDetail } from '@/lib/api/comic'
import { SINGLE_CHAPTER_TITLE } from '@/lib/comic'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { resolveReaderChapterInfo } from './chapter-utils'
import type { ReaderSearch } from './types'

export function useReaderChapterInfo({
  comicId,
  search
}: {
  comicId: string
  search: ReaderSearch
}) {
  const albumId = safeAlbumId(search.albumId) || comicId
  const albumDetail = useQuery({
    queryKey: queryKeys.comicDetail(albumId),
    queryFn: () => getComicDetail(albumId),
    enabled: albumId.length > 0,
    staleTime: CACHE.READER_STALE_TIME,
    gcTime: CACHE.READER_GC_TIME,
    retry: false,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const chapters = albumDetail.data?.comic.chapters
  const chapterInfo = useMemo(
    () =>
      resolveReaderChapterInfo({
        currentReadId: comicId,
        chapters: chapters ?? []
      }),
    [chapters, comicId]
  )
  const title = safeTrim(albumDetail.data?.comic.title)
  const author = albumDetail.data?.comic.authors.join(' / ') ?? ''
  const coverUrl = albumDetail.data?.comic.image ?? ''
  const chapterTitle =
    chapterInfo.chapterTitle ||
    (albumDetail.data?.comic.chapters.length === 0 ? SINGLE_CHAPTER_TITLE : '')

  return {
    albumId,
    title,
    author,
    coverUrl,
    chapter: chapterTitle,
    chapters: chapterInfo.chapters,
    previousChapter: chapterInfo.previousChapter,
    nextChapter: chapterInfo.nextChapter
  }
}

function safeTrim(value: string | null | undefined) {
  return typeof value === 'string' ? value.trim() : ''
}

function safeAlbumId(value: string | null | undefined) {
  const albumId = safeTrim(value)

  return albumId === '0' ? '' : albumId
}
