import { apiClient } from './client'

export type ComicReadManifestResult = {
  endpoint: string
  readId: string
  pageCount: number
  cacheLimitBytes: number
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

export type ReaderCacheStatsResult = {
  cacheDir: string
  totalBytes: number
  fileCount: number
  cacheLimitBytes: number
  cacheTrimBytes: number
}

export async function getComicReadManifest({
  readId,
  endpoint = null
}: {
  readId: string
  endpoint?: string | null
}): Promise<ComicReadManifestResult> {
  const result = await apiClient.get<{
    chapter_id: string
    pages: Array<{
      index: number
      name: string
      url: string
    }>
  }>(`/api/reader/${readId}/manifest`, {
    endpoint: endpoint || 'https://www.cdnhjk.net'
  })

  return {
    endpoint: endpoint || '',
    readId: result.chapter_id,
    pageCount: result.pages.length,
    cacheLimitBytes: 0
  }
}

export async function getComicReadPage({
  readId,
  index,
  endpoint = null,
  requestOrigin: _requestOrigin = null,
  cacheLimitBytes: _cacheLimitBytes = null
}: {
  readId: string
  index: number
  endpoint?: string | null
  requestOrigin?: ReaderPageRequestOrigin | null
  cacheLimitBytes?: number | null
}): Promise<ComicReadPageResult> {
  // HTTP 模式：直接返回图片 URL（后端会处理解扰和转码）
  const endpointParam = endpoint || 'https://www.cdnhjk.net'
  const imageUrl = `/api/reader/${readId}/pages/${index}?endpoint=${encodeURIComponent(endpointParam)}`

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

export async function getReaderCacheStats(
  _cacheLimitBytes: number | null = null
): Promise<ReaderCacheStatsResult> {
  // HTTP 模式：无缓存统计
  return {
    cacheDir: '',
    totalBytes: 0,
    fileCount: 0,
    cacheLimitBytes: 0,
    cacheTrimBytes: 0
  }
}

export async function clearReaderCache(
  _cacheLimitBytes: number | null = null
): Promise<ReaderCacheStatsResult> {
  // HTTP 模式：无缓存
  return {
    cacheDir: '',
    totalBytes: 0,
    fileCount: 0,
    cacheLimitBytes: 0,
    cacheTrimBytes: 0
  }
}

export async function openReaderCacheDir(): Promise<void> {
  // HTTP 模式：无操作
  console.log('Cache directory not available in HTTP mode')
}

export function readerFileSrc(path: string) {
  // HTTP 模式：直接返回路径（已经是 HTTP URL）
  return path
}
