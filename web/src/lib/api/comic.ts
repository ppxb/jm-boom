import { apiClient } from './client'
import type { FeedComic } from './home'

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
  isFavorite: boolean
  liked: boolean
  relatedList: RelatedComic[]
  series: ComicChapter[]
  seriesId: string
  price: number
  purchased: boolean
  image: string
}

export type ComicDetailResult = {
  endpoint: string
  comic: ComicDetail
}

export type FavoriteToggleResult = {
  endpoint: string
  favorited: boolean
}

export type FavoriteFolder = {
  id: string
  name: string
}

export type FavoriteListResult = {
  endpoint: string
  page: number
  total: number
  hasMore: boolean
  folders: FavoriteFolder[]
  items: FeedComic[]
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
  endpoint: string
  page: number
  total: number
  comments: ComicComment[]
}

export async function getComicDetail(
  comicId: string,
  endpoint: string | null = null
): Promise<ComicDetailResult> {
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
    is_favorite: boolean
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
    endpoint: endpoint || '',
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
      isFavorite: result.is_favorite,
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
  comicId: _comicId,
  page = 1,
  endpoint = null
}: {
  comicId: string
  page?: number
  endpoint?: string | null
}): Promise<ComicCommentsResult> {
  // TODO: 后端未实现评论 API
  return {
    endpoint: endpoint || '',
    page,
    total: 0,
    comments: []
  }
}

export async function toggleComicFavorite({
  comicId: _comicId,
  currentFavorite: _currentFavorite,
  endpoint: _endpoint = null
}: {
  comicId: string
  currentFavorite: boolean
  endpoint?: string | null
}): Promise<FavoriteToggleResult> {
  // TODO: 后端未实现收藏 API
  throw new Error('Favorite toggle not implemented in HTTP mode')
}

export async function getFavoriteComics({
  page = 1,
  folderId: _folderId = '',
  order: _order = 'mr',
  endpoint = null
}: {
  page?: number
  folderId?: string
  order?: string
  endpoint?: string | null
} = {}): Promise<FavoriteListResult> {
  // TODO: 后端未实现收藏列表 API
  return {
    endpoint: endpoint || '',
    page,
    total: 0,
    hasMore: false,
    folders: [],
    items: []
  }
}
