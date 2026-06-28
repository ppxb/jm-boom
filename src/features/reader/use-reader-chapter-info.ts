import { useQuery } from '@tanstack/react-query'
import { useMemo } from 'react'

import { getComicDetail } from '@/lib/api/comic'
import { useSettingsStore } from '@/stores/settings-store'
import { READER_GC_TIME, READER_STALE_TIME } from './constants'
import { resolveCurrentChapterTitle, resolveNextChapter, toNextChapter } from './chapter-utils'
import type { ReaderSearch } from './types'

export function useReaderChapterInfo({ comicId, search }: { comicId: string; search: ReaderSearch }) {
  const endpoint = useSettingsStore(state => state.api)
  const albumId = safeAlbumId(search.albumId)
  const title = safeTrim(search.title)
  const searchChapter = safeTrim(search.chapter)
  const fallbackNextChapter = useMemo(
    () => toNextChapter(search.nextId, search.nextChapter),
    [search.nextId, search.nextChapter]
  )
  const albumDetail = useQuery({
    queryKey: ['jm-comic-detail', endpoint, albumId],
    queryFn: () => getComicDetail(albumId, endpoint),
    enabled: albumId.length > 0,
    staleTime: READER_STALE_TIME,
    gcTime: READER_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const chapters = albumDetail.data?.comic.series ?? []
  const nextChapter = useMemo(
    () =>
      resolveNextChapter({
        currentReadId: comicId,
        chapters,
        fallback: fallbackNextChapter
      }),
    [chapters, comicId, fallbackNextChapter]
  )
  const chapter = useMemo(
    () =>
      resolveCurrentChapterTitle({
        currentReadId: comicId,
        chapters,
        fallback: searchChapter
      }),
    [chapters, comicId, searchChapter]
  )
  const author = albumDetail.data?.comic.author.join(' / ') ?? ''
  const coverUrl = albumDetail.data?.comic.image ?? ''

  return { albumId, title, author, coverUrl, chapter, nextChapter }
}

function safeTrim(value: string | null | undefined) {
  return typeof value === 'string' ? value.trim() : ''
}

function safeAlbumId(value: string | null | undefined) {
  const albumId = safeTrim(value)

  return albumId === '0' ? '' : albumId
}
