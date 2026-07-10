import { convertFileSrc } from '@tauri-apps/api/core'
import { tauriInvoke } from './tauri'

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
  return tauriInvoke<ComicReadManifestResult>('get_comic_read_manifest', {
    readId,
    endpoint
  })
}

export async function getComicReadPage({
  readId,
  index,
  endpoint = null,
  requestOrigin = null,
  cacheLimitBytes = null
}: {
  readId: string
  index: number
  endpoint?: string | null
  requestOrigin?: ReaderPageRequestOrigin | null
  cacheLimitBytes?: number | null
}): Promise<ComicReadPageResult> {
  return tauriInvoke<ComicReadPageResult>('get_comic_read_page', {
    readId,
    index,
    endpoint,
    requestOrigin,
    cacheLimitBytes
  })
}

export async function getReaderCacheStats(
  cacheLimitBytes: number | null = null
): Promise<ReaderCacheStatsResult> {
  return tauriInvoke<ReaderCacheStatsResult>('get_reader_cache_stats', { cacheLimitBytes })
}

export async function clearReaderCache(
  cacheLimitBytes: number | null = null
): Promise<ReaderCacheStatsResult> {
  return tauriInvoke<ReaderCacheStatsResult>('clear_reader_cache', { cacheLimitBytes })
}

export async function openReaderCacheDir(): Promise<void> {
  return tauriInvoke<void>('open_reader_cache_dir')
}

export function readerFileSrc(path: string) {
  return convertFileSrc(path)
}
