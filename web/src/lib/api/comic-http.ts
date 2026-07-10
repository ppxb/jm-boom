import { apiClient } from './client'

// ============== Types ==============

export type RelatedComic = {
  id: string
  name: string
  author: string
  image: string
}

export type ComicChapter = {
  id: string
  name: string
  sort: string
}

export type ComicDetail = {
  id: string
  name: string
  author: string[]
  description: string
  tags: string[]
  actors: string[]
  works: string[]
  total_views: number
  likes: number
  comment_total: number
  is_favorite: boolean
  liked: boolean
  related_list: RelatedComic[]
  series: ComicChapter[]
}

// ============== API Functions ==============

export async function getComicDetail(comicId: string): Promise<ComicDetail> {
  return apiClient.get<ComicDetail>(`/api/comics/${comicId}`)
}

export async function getComicChapters(comicId: string): Promise<ComicChapter[]> {
  return apiClient.get<ComicChapter[]>(`/api/comics/${comicId}/chapters`)
}
