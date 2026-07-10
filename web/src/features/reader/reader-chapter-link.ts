import type { ReaderSearch } from './types'

export function toReaderChapterSearch({ albumId }: { albumId: string }): ReaderSearch {
  return { albumId }
}
