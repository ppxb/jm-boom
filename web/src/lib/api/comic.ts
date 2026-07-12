import { apiClient } from './client'

export type RelatedComic = {
  id: string
  title: string
  author: string
  image: string
}

export type ComicChapter = {
  id: string
  title: string
  sort: string
}

export type ComicDetail = {
  id: string
  title: string
  author: string[]
  description: string
  totalViews: number
  likes: number
  commentTotal: number
  tags: string[]
  actors: string[]
  works: string[]
  relatedList: RelatedComic[]
  series: ComicChapter[]
  image: string
}

export type ComicDetailResult = {
  comic: ComicDetail
}

export type ComicComment = {
  id: string
  comicId?: string | null
  userId: string
  username: string
  nickname: string
  content: string
  likeCount: number
  time: string
  updatedAt: string
  avatar: string
  parentId: string
  spoiler: boolean
  replies: ComicComment[]
}

export type ComicCommentsResult = {
  page: number
  total: number
  comments: ComicComment[]
}

export async function getComicDetail(comicId: string): Promise<ComicDetailResult> {
  const result = await apiClient.get<{
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
    related_list: Array<{
      id: string
      name: string
      author: string
      image: string
    }>
    series: Array<{
      id: string
      name: string
      sort: string
    }>
    image: string
  }>(`/api/comics/${comicId}`)

  return {
    comic: {
      id: result.id,
      title: result.name,
      author: normalizeTextList(result.author),
      description: result.description,
      totalViews: result.total_views,
      likes: result.likes,
      commentTotal: result.comment_total,
      tags: normalizeTextList(result.tags),
      actors: normalizeTextList(result.actors),
      works: normalizeTextList(result.works),
      relatedList: result.related_list.map(r => ({
        id: r.id,
        title: r.name,
        author: r.author,
        image: r.image
      })),
      series: result.series.map(s => ({
        id: s.id,
        title: s.name,
        sort: s.sort
      })),
      image: result.image
    }
  }
}

export async function getComicComments({
  comicId,
  page = 1
}: {
  comicId: string
  page?: number
}): Promise<ComicCommentsResult> {
  return apiClient.get(`/api/comics/${comicId}/comments`, { page })
}

function normalizeTextList(items: string[]) {
  const seen = new Set<string>()
  const normalizedItems: string[] = []

  for (const item of items) {
    const normalized = item.trim()

    if (normalized.length === 0 || seen.has(normalized)) {
      continue
    }

    seen.add(normalized)
    normalizedItems.push(normalized)
  }

  return normalizedItems
}
