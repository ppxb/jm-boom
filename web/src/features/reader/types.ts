export type ReaderSearch = {
  albumId: string
  page?: number
}

export type ReaderChapterItem = {
  id: string
  title: string
}

export type ReaderWindowPage = {
  index: number
  src: string
  key: string
  chapterId: string
  localIndex: number
  width?: number
  height?: number
}
import type { ComicReadManifestPage } from '@/lib/api/reader'

export type ReaderPageItem = ComicReadManifestPage & {
  key: string
  chapterId: string
  localIndex: number
}

export type ReaderBoundary = {
  chapter: ReaderChapterItem | null
  loading: boolean
  error: string | null
}
