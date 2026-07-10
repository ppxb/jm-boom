import { tauriInvoke } from './tauri'
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
  return tauriInvoke<ComicDetailResult>('get_comic_detail', {
    comicId,
    endpoint
  })
}

export async function getComicComments({
  comicId,
  page = 1,
  endpoint = null
}: {
  comicId: string
  page?: number
  endpoint?: string | null
}): Promise<ComicCommentsResult> {
  return tauriInvoke<ComicCommentsResult>('get_comic_comments', {
    comicId,
    page,
    endpoint
  })
}

export async function toggleComicFavorite({
  comicId,
  currentFavorite,
  endpoint = null
}: {
  comicId: string
  currentFavorite: boolean
  endpoint?: string | null
}): Promise<FavoriteToggleResult> {
  return tauriInvoke<FavoriteToggleResult>('toggle_comic_favorite', {
    comicId,
    currentFavorite,
    endpoint
  })
}

export async function getFavoriteComics({
  page = 1,
  folderId = '',
  order = 'mr',
  endpoint = null
}: {
  page?: number
  folderId?: string
  order?: string
  endpoint?: string | null
} = {}): Promise<FavoriteListResult> {
  return tauriInvoke<FavoriteListResult>('get_favorite_comics', {
    page,
    folderId,
    order,
    endpoint
  })
}
