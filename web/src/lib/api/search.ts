import { tauriInvoke } from './tauri'

export type StringMap = Record<string, unknown>

export type ImageItem = {
  id: string
  url: string
  name: string
  path: string
  extern: StringMap
}

export type MetadataListItem = {
  type: string
  name: string
  value: string[]
}

export type PagingInfo = {
  page: number
  pages: number
  total: number
  hasReachedMax: boolean
}

export type ComicListItem = {
  source: string
  id: string
  title: string
  subtitle: string
  finished: boolean
  likesCount: number
  viewsCount: number
  updatedAt: string
  cover: ImageItem
  metadata: MetadataListItem[]
  raw: StringMap
  extern: StringMap
}

export type SearchResultContract = {
  source: string
  extern: StringMap
  scheme: {
    version: '1.0.0'
    type: 'searchResult'
    source: string
    list: string
  }
  data: {
    paging: PagingInfo
    items: ComicListItem[]
  }
  paging: PagingInfo
  items: ComicListItem[]
}

export type SearchComicParams = {
  keyword: string
  page?: number
  extern?: StringMap | null
  endpoint?: string | null
}

export async function searchComic({
  keyword,
  page = 1,
  extern = null,
  endpoint = null
}: SearchComicParams): Promise<SearchResultContract> {
  const normalizedKeyword = keyword.trim()

  if (normalizedKeyword.length === 0) {
    return emptySearchResult(page, extern)
  }

  return withTimeout(
    tauriInvoke<SearchResultContract>(
      'search_comics',
      {
        keyword: normalizedKeyword,
        page,
        externPayload: extern,
        endpoint
      },
      'Search needs the Tauri desktop runtime. Start the app with the Tauri command.'
    ),
    15000
  )
}

function emptySearchResult(page: number, extern: StringMap | null): SearchResultContract {
  const paging = {
    page,
    pages: page,
    total: 0,
    hasReachedMax: true
  }
  const items: ComicListItem[] = []

  return {
    source: 'bf99008d-010b-4f17-ac7c-61a9b57dc3d9',
    extern: extern ?? { sortBy: 1 },
    scheme: {
      version: '1.0.0',
      type: 'searchResult',
      source: 'bf99008d-010b-4f17-ac7c-61a9b57dc3d9',
      list: 'comicGrid'
    },
    data: {
      paging,
      items
    },
    paging,
    items
  }
}

function withTimeout<T>(promise: Promise<T>, timeoutMs: number) {
  return new Promise<T>((resolve, reject) => {
    const timeoutId = window.setTimeout(() => {
      reject(new Error('Search timed out. The current API endpoints may be unreachable.'))
    }, timeoutMs)

    promise.then(
      value => {
        window.clearTimeout(timeoutId)
        resolve(value)
      },
      error => {
        window.clearTimeout(timeoutId)
        reject(error)
      }
    )
  })
}
