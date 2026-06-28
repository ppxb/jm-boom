import { convertFileSrc, invoke } from '@tauri-apps/api/core'

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

export type ComicReadChapterCacheResult = {
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
  endpoint = null
}: {
  readId: string
  endpoint?: string | null
}): Promise<ComicReadManifestResult> {
  ensureTauriRuntime()

  return invoke<ComicReadManifestResult>('get_comic_read_manifest', {
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
  requestOrigin?: 'visible' | 'chapter_cache' | null
  cacheLimitBytes?: number | null
}): Promise<ComicReadPageResult> {
  ensureTauriRuntime()

  return invoke<ComicReadPageResult>('get_comic_read_page', {
    readId,
    index,
    endpoint,
    requestOrigin,
    cacheLimitBytes
  })
}

export async function cacheComicReadChapter({
  readId,
  endpoint = null,
  requestOrigin = null,
  cacheLimitBytes = null
}: {
  readId: string
  endpoint?: string | null
  requestOrigin?: 'visible' | 'chapter_cache' | null
  cacheLimitBytes?: number | null
}): Promise<ComicReadChapterCacheResult> {
  ensureTauriRuntime()

  return invoke<ComicReadChapterCacheResult>('cache_comic_read_chapter', {
    readId,
    endpoint,
    requestOrigin,
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
