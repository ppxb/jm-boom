import { apiClient } from './client'

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
}

export async function searchComic({
  keyword,
  page = 1,
  extern = null
}: SearchComicParams): Promise<SearchResultContract> {
  const normalizedKeyword = keyword.trim()

  if (normalizedKeyword.length === 0) {
    return emptySearchResult(page, extern)
  }

  // 调用 HTTP API (不再传递 endpoint，由后端管理)
  const result = await apiClient.get<{
    total: number
    content: Array<{
      id: string
      name: string
      author: string
      description: string
      image: string
      tags: string[]
      likes: number
      views: number
      is_favorite: boolean
      liked: boolean
    }>
    redirect_aid?: string | null
  }>('/api/search', {
    keyword: normalizedKeyword,
    page,
    sortBy: Number(extern?.sortBy ?? 1)
  })

  // 转换为前端格式
  const paging = {
    page,
    pages: Math.ceil(result.total / 80),
    total: result.total,
    hasReachedMax: page >= Math.ceil(result.total / 80)
  }

  const items: ComicListItem[] = result.content.map(comic => ({
    source: 'jm-boom-http',
    id: comic.id,
    title: comic.name,
    subtitle: '',
    finished: false,
    likesCount: comic.likes,
    viewsCount: comic.views,
    updatedAt: Date.now().toString(),
    cover: {
      id: comic.id,
      url: comic.image,
      name: comic.name,
      path: comic.image,
      extern: {}
    },
    metadata: [
      {
        type: 'author',
        name: '作者',
        value: [comic.author]
      },
      {
        type: 'tags',
        name: '标签',
        value: comic.tags
      }
    ],
    raw: {
      description: comic.description || '',
      author: comic.author
    },
    extern: {}
  }))

  return {
    source: 'jm-boom-http',
    extern: extern ?? { sortBy: 1 },
    scheme: {
      version: '1.0.0',
      type: 'searchResult',
      source: 'jm-boom-http',
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

function emptySearchResult(page: number, extern: StringMap | null): SearchResultContract {
  const paging = {
    page,
    pages: page,
    total: 0,
    hasReachedMax: true
  }
  const items: ComicListItem[] = []

  return {
    source: 'jm-boom-http',
    extern: extern ?? { sortBy: 1 },
    scheme: {
      version: '1.0.0',
      type: 'searchResult',
      source: 'jm-boom-http',
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
