import { apiClient } from './client'
import type { ComicDetail } from '@/domain/comic'
import type { ReadingHistoryItem } from './history'

export type ComicDetailResult = {
  comic: ComicDetail
}

export type ComicStateResult = {
  isFavorite: boolean
  history: ReadingHistoryItem | null
}

type ComicDetailResponse = {
  id: string
  title: string
  description: string
  image: string
  authors: string[]
  tags: string[]
  actors: string[]
  works: string[]
  totalViews: number
  likes: number
  commentCount: number
  relatedComics: RelatedComicResponse[]
  chapters: ComicChapterResponse[]
}

type RelatedComicResponse = {
  id: string
  title: string
  author: string
  image: string
}

type ComicChapterResponse = {
  id: string
  title: string
  sort: string
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

type ComicCommentResponse = Omit<ComicComment, 'replies'> & {
  replies: ComicCommentResponse[]
}

type ComicCommentsResponse = Omit<ComicCommentsResult, 'comments'> & {
  comments: ComicCommentResponse[]
}

export async function getComicDetail(comicId: string): Promise<ComicDetailResult> {
  const response = await apiClient.get<ComicDetailResponse>(`/api/comics/${comicId}`)

  return {
    comic: mapComicDetail(response)
  }
}

export function getComicState(comicId: string): Promise<ComicStateResult> {
  return apiClient.get(`/api/comics/${comicId}/state`)
}

export async function getComicComments({
  comicId,
  page = 1
}: {
  comicId: string
  page?: number
}): Promise<ComicCommentsResult> {
  const response = await apiClient.get<ComicCommentsResponse>(`/api/comics/${comicId}/comments`, {
    page
  })

  return {
    ...response,
    comments: response.comments.map(mapComicComment)
  }
}

function mapComicComment(comment: ComicCommentResponse): ComicComment {
  return {
    ...comment,
    content: htmlToText(comment.content),
    replies: comment.replies.map(mapComicComment)
  }
}

function htmlToText(value: string) {
  const { body } = new DOMParser().parseFromString(value, 'text/html')
  return body.textContent?.trim() ?? ''
}

function mapComicDetail(response: ComicDetailResponse): ComicDetail {
  return {
    id: response.id,
    title: response.title,
    description: response.description,
    image: response.image,
    authors: response.authors,
    tags: response.tags,
    actors: response.actors,
    works: response.works,
    totalViews: response.totalViews,
    likes: response.likes,
    commentCount: response.commentCount,
    relatedComics: response.relatedComics.map(related => ({
      id: related.id,
      title: related.title,
      author: related.author,
      image: related.image
    })),
    chapters: response.chapters.map(chapter => ({
      id: chapter.id,
      title: chapter.title,
      sort: chapter.sort
    }))
  }
}
