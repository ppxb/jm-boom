import { convertFileSrc, invoke } from '@tauri-apps/api/core'

export type ComicReadManifestResult = {
  endpoint: string
  readId: string
  shunt: string
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

export type ComicReadPrefetchResult = {
  requested: number
  completed: number
}

export type ReaderCacheStatsResult = {
  cacheDir: string
  totalBytes: number
  fileCount: number
  cacheLimitBytes: number
  cacheTrimBytes: number
}

export async function getComicReadManifest({
  readId,
  shunt = null,
  endpoint = null
}: {
  readId: string
  shunt?: string | null
  endpoint?: string | null
}): Promise<ComicReadManifestResult> {
  ensureTauriRuntime()

  return invoke<ComicReadManifestResult>('get_comic_read_manifest', {
    readId,
    shunt,
    endpoint
  })
}

export async function getComicReadPage({
  readId,
  index,
  shunt = null,
  endpoint = null,
  cacheLimitBytes = null
}: {
  readId: string
  index: number
  shunt?: string | null
  endpoint?: string | null
  cacheLimitBytes?: number | null
}): Promise<ComicReadPageResult> {
  ensureTauriRuntime()

  return invoke<ComicReadPageResult>('get_comic_read_page', {
    readId,
    index,
    shunt,
    endpoint,
    cacheLimitBytes
  })
}

export async function prefetchComicReadPages({
  readId,
  centerIndex,
  radius,
  shunt = null,
  endpoint = null,
  cacheLimitBytes = null
}: {
  readId: string
  centerIndex: number
  radius: number
  shunt?: string | null
  endpoint?: string | null
  cacheLimitBytes?: number | null
}): Promise<ComicReadPrefetchResult> {
  ensureTauriRuntime()

  return invoke<ComicReadPrefetchResult>('prefetch_comic_read_pages', {
    readId,
    centerIndex,
    radius,
    shunt,
    endpoint,
    cacheLimitBytes
  })
}

export async function getReaderCacheStats(
  cacheLimitBytes: number | null = null
): Promise<ReaderCacheStatsResult> {
  ensureTauriRuntime()

  return invoke<ReaderCacheStatsResult>('get_reader_cache_stats', { cacheLimitBytes })
}

export async function clearReaderCache(
  cacheLimitBytes: number | null = null
): Promise<ReaderCacheStatsResult> {
  ensureTauriRuntime()

  return invoke<ReaderCacheStatsResult>('clear_reader_cache', { cacheLimitBytes })
}

export async function openReaderCacheDir(): Promise<void> {
  ensureTauriRuntime()

  return invoke('open_reader_cache_dir')
}

export function readerFileSrc(path: string) {
  return convertFileSrc(path)
}

function ensureTauriRuntime() {
  if (!('__TAURI_INTERNALS__' in window)) {
    throw new Error('This content needs the Tauri desktop runtime.')
  }
}
