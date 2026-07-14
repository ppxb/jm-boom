import { apiClient } from './client'
import { mapComicSummary, type ComicSummaryResponse } from './comic-summary'
import type { ComicSummary } from '@/domain/comic'

export type PagingInfo = {
  page: number
  pages: number
  total: number
  hasReachedMax: boolean
}

export type SearchResult = {
  paging: PagingInfo
  items: ComicSummary[]
}

export type SearchComicParams = {
  keyword: string
  page?: number
  sortBy?: number
}

type SearchResponse = Omit<SearchResult, 'items'> & {
  items: ComicSummaryResponse[]
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

  const response = await apiClient.get<SearchResponse>('/api/search', {
    keyword: normalizedKeyword,
    page,
    sortBy
  })
  return { ...response, items: response.items.map(mapComicSummary) }
}
