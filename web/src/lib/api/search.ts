import { apiClient } from './client'

export type PagingInfo = {
  page: number
  pages: number
  total: number
  hasReachedMax: boolean
}

export type SearchComicItem = {
  id: string
  title: string
  author: string
  description: string
  image: string
  tags: string[]
  updatedAt: number | null
}

export type SearchResult = {
  paging: PagingInfo
  items: SearchComicItem[]
}

export type SearchComicParams = {
  keyword: string
  page?: number
  sortBy?: number
}

export async function searchComic({
  keyword,
  page = 1,
  sortBy = 1
}: SearchComicParams): Promise<SearchResult> {
  const normalizedKeyword = keyword.trim()

  if (normalizedKeyword.length === 0) {
    return {
      paging: {
        page,
        pages: 0,
        total: 0,
        hasReachedMax: true
      },
      items: []
    }
  }

  return apiClient.get<SearchResult>('/api/search', {
    keyword: normalizedKeyword,
    page,
    sortBy
  })
}
