import { apiClient } from './client'

export type ComicReadManifestResult = {
  readId: string
  pageCount: number
}

export type ComicReadPageResult = {
  readId: string
  index: number
  path: string
  width: number
  height: number
  aspectRatio: number
  isCached: boolean
}

type ReaderPageRequestOrigin = 'visible' | 'prefetch'

export async function getComicReadManifest({
  readId
}: {
  readId: string
}): Promise<ComicReadManifestResult> {
  const result = await apiClient.get<{
    chapter_id: string
    pages: Array<{
      index: number
      name: string
      url: string
    }>
  }>(`/api/reader/${readId}/manifest`)

  return {
    readId: result.chapter_id,
    pageCount: result.pages.length
  }
}

export async function getComicReadPage({
  readId,
  index,
  requestOrigin: _requestOrigin = null
}: {
  readId: string
  index: number
  requestOrigin?: ReaderPageRequestOrigin | null
}): Promise<ComicReadPageResult> {
  const imageUrl = `/api/reader/${readId}/pages/${index}`

  return {
    readId,
    index,
    path: imageUrl,
    width: 800,
    height: 1200,
    aspectRatio: 800 / 1200,
    isCached: false
  }
}

export function readerFileSrc(path: string) {
  // HTTP 模式：直接返回路径（已经是 HTTP URL）
  return path
}
