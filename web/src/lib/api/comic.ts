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
  liked: boolean
  relatedList: RelatedComic[]
  series: ComicChapter[]
  seriesId: string
  price: number
  purchased: boolean
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
    liked: boolean
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
      author: result.author,
      description: result.description,
      totalViews: result.total_views,
      likes: result.likes,
      commentTotal: result.comment_total,
      tags: result.tags,
      actors: result.actors,
      works: result.works,
      liked: result.liked,
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
      seriesId: '',
      price: 0,
      purchased: true,
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
