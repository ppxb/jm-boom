import { useQuery } from '@tanstack/react-query'
import { useMemo } from 'react'

import { getComicDetail } from '@/lib/api/comic'
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
  const albumId = safeAlbumId(search.albumId)
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
  const chapters = albumDetail.data?.comic.series
  const chapterInfo = useMemo(
    () =>
      resolveReaderChapterInfo({
        currentReadId: comicId,
        chapters: chapters ?? []
      }),
    [chapters, comicId]
  )
  const title = safeTrim(albumDetail.data?.comic.title)
  const author = albumDetail.data?.comic.author.join(' / ') ?? ''
  const coverUrl = albumDetail.data?.comic.image ?? ''

  return {
    albumId,
    title,
    author,
    coverUrl,
    chapter: chapterInfo.chapterTitle,
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
