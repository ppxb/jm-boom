import { apiClient, resolveApiUrl } from './client'
import type { ComicReadManifestPage } from './reader'

export type SourceInfo = {
  id: string
  name: string
  version: number
  altNames: string[]
  url: string | null
  urls: string[]
  languages: string[]
  contentRating: number | null
  minAppVersion: string | null
  maxAppVersion: string | null
}

export type SourceCapabilities = {
  providesHome: boolean
  providesListings: boolean
  dynamicListings: boolean
  dynamicFilters: boolean
  dynamicSettings: boolean
  providesImageRequests: boolean
  processesPages: boolean
  providesPageDescriptions: boolean
  providesAlternateCovers: boolean
  providesBaseUrl: boolean
  handlesNotifications: boolean
  handlesDeepLinks: boolean
  handlesBasicLogin: boolean
  handlesWebLogin: boolean
  handlesMigration: boolean
  sendsPartialResults: boolean
  usesNetwork: boolean
  usesHtml: boolean
  usesCanvas: boolean
  usesDefaults: boolean
  usesJavascript: boolean
}

export type InstalledSource = {
  info: SourceInfo
  capabilities: SourceCapabilities
  listings: SourceListing[]
  filterCount: number
  settingCount: number
}

export type SourceListing = {
  id: string
  name: string
  kind: 'default' | 'list'
}

export type AvailableSource = {
  id: string
  name: string
  version: number
  iconUrl: string | null
  downloadUrl: string | null
  languages: string[]
  contentRating: number
  installedVersion: number | null
}

export type SourceManga = {
  key: string
  title: string
  cover: string | null
  artists: string[] | null
  authors: string[] | null
  description: string | null
  url: string | null
  tags: string[] | null
  status: string
  contentRating: string
  viewer: string
  updateStrategy: string
  nextUpdateTime: number | null
  chapters: SourceChapter[] | null
}

export type SourceChapter = {
  key: string
  title: string | null
  chapterNumber: number | null
  volumeNumber: number | null
  dateUploaded: number | null
  scanlators: string[] | null
  url: string | null
  language: string | null
  thumbnail: string | null
  locked: boolean
}

export type SourceSearchResponse = {
  sourceId: string
  result: {
    entries: SourceManga[]
    hasNextPage: boolean
  }
}

export type SourceDetailResponse = {
  sourceId: string
  manga: SourceManga
}

export type SourcePage = {
  content:
    | {
        type: 'remote'
        data: {
          url: string
          context: Record<string, string> | null
        }
      }
    | {
        type: 'text'
        data: {
          text: string
        }
      }
    | {
        type: 'archive'
        data: {
          url: string
          path: string
        }
      }
  thumbnail: string | null
  hasDescription: boolean
  description: string | null
}

export type SourcePagesResponse = {
  sourceId: string
  pages: SourcePage[]
}

export type SourceListingResponse = {
  sourceId: string
  result: {
    entries: SourceManga[]
    hasNextPage: boolean
  }
}

export type SourceSearchGroup = {
  source: InstalledSource
  entries: SourceManga[]
  hasNextPage: boolean
  error: string | null
}

export function getInstalledSources() {
  return apiClient.get<InstalledSource[]>('/api/sources')
}

export function getSourceCatalog(refresh = false) {
  return apiClient.get<AvailableSource[]>('/api/sources/catalog', { refresh })
}

export function installSource(sourceId: string) {
  return apiClient.post<InstalledSource>(
    `/api/sources/catalog/${encodeURIComponent(sourceId)}/install`
  )
}

export async function searchSource(
  source: InstalledSource,
  query: string,
  page: number
): Promise<SourceSearchGroup> {
  try {
    const response = await apiClient.post<SourceSearchResponse>(
      `/api/sources/${encodeURIComponent(source.info.id)}/search`,
      { query: query || null, page, filters: [] }
    )
    return {
      source,
      entries: response.result.entries,
      hasNextPage: response.result.hasNextPage,
      error: null
    }
  } catch (error) {
    return {
      source,
      entries: [],
      hasNextPage: false,
      error: error instanceof Error ? error.message : String(error)
    }
  }
}

export function searchInstalledSources(
  sources: InstalledSource[],
  query: string,
  page: number
) {
  return Promise.all(sources.map(source => searchSource(source, query, page)))
}

export async function getSourceManga(sourceId: string, manga: SourceManga) {
  const response = await apiClient.post<SourceDetailResponse>(
    `/api/sources/${encodeURIComponent(sourceId)}/manga`,
    {
      manga,
      needsDetails: true,
      needsChapters: true
    }
  )
  return response.manga
}

export async function getSourceReaderPages(
  sourceId: string,
  manga: SourceManga,
  chapter: SourceChapter
): Promise<ComicReadManifestPage[]> {
  const response = await apiClient.post<SourcePagesResponse>(
    `/api/sources/${encodeURIComponent(sourceId)}/pages`,
    { manga, chapter }
  )
  const unsupportedPage = response.pages.find(page => page.content.type !== 'remote')
  if (unsupportedPage) {
    throw new Error(`当前阅读器暂不支持 ${unsupportedPage.content.type} 类型页面`)
  }
  return response.pages.map((page, index) => {
    const content = page.content
    if (content.type !== 'remote') {
      throw new Error('漫画源页面类型在解析过程中发生变化')
    }
    return {
      index,
      name: String(index + 1).padStart(3, '0'),
      path: resolveApiUrl(content.data.url)
    }
  })
}

export async function getSourceListing(
  sourceId: string,
  listing: SourceListing,
  page = 1
) {
  const response = await apiClient.post<SourceListingResponse>(
    `/api/sources/${encodeURIComponent(sourceId)}/listings`,
    { listing, page }
  )
  return response.result
}

export function createSourceMangaStub(key: string): SourceManga {
  return {
    key,
    title: key,
    cover: null,
    artists: null,
    authors: null,
    description: null,
    url: null,
    tags: null,
    status: 'unknown',
    contentRating: 'unknown',
    viewer: 'unknown',
    updateStrategy: 'always',
    nextUpdateTime: null,
    chapters: null
  }
}

export function createSourceChapterStub(key: string): SourceChapter {
  return {
    key,
    title: null,
    chapterNumber: null,
    volumeNumber: null,
    dateUploaded: null,
    scanlators: null,
    url: null,
    language: null,
    thumbnail: null,
    locked: false
  }
}

export function mapSourceManga(manga: SourceManga) {
  return {
    id: manga.key,
    title: manga.title,
    author: [...(manga.authors ?? []), ...(manga.artists ?? [])].join(' / '),
    description: manga.description ?? '',
    image: manga.cover ?? '',
    tags: manga.tags ?? []
  }
}
