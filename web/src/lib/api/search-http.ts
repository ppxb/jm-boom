import { apiClient } from './client'

// ============== Types ==============

export type Comic = {
  id: string
  name: string
  author: string
  description?: string
  image: string
  tags: string[]
  likes: number
  views: number
  is_favorite: boolean
  liked: boolean
}

export type SearchResult = {
  total: number
  content: Comic[]
  redirect_aid?: string
}

// ============== API Functions ==============

export async function searchComics({
  keyword,
  page = 1,
}: {
  keyword: string
  page?: number
}): Promise<SearchResult> {
  return apiClient.get<SearchResult>('/api/search', {
    keyword,
    page,
  })
}
