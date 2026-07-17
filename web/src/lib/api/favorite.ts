import type { ComicSummary } from '@/domain/comic'
import { apiClient } from './client'

const LOCAL_FAVORITES_KEY = 'jm-boom-local-favorites-v2'

export type FavoriteItem = ComicSummary & {
  favoritedAt: number
}

export type FavoriteListResult = {
  items: FavoriteItem[]
}

export async function listFavorites(): Promise<FavoriteListResult> {
  const result = await apiClient.get<FavoriteListResult>('/api/favorites')
  const localItems = readLocalFavorites()

  if (localItems.length === 0) {
    return result
  }

  const imported = await apiClient.post<FavoriteListResult>('/api/favorites/import', {
    items: localItems
  })
  try {
    localStorage.removeItem(LOCAL_FAVORITES_KEY)
  } catch {
    // Import is idempotent, so keeping the old value only causes a harmless retry.
  }
  return imported
}

export function addFavorite(comic: ComicSummary): Promise<FavoriteListResult> {
  return apiClient.put<FavoriteListResult>(`/api/favorites/${comic.id}`, {
    title: comic.title,
    author: comic.author,
    description: comic.description,
    image: comic.image,
    tags: comic.tags
  })
}

export function removeFavorite(comicId: string): Promise<FavoriteListResult> {
  return apiClient.delete<FavoriteListResult>(`/api/favorites/${comicId}`)
}

export function clearFavorites(): Promise<FavoriteListResult> {
  return apiClient.delete<FavoriteListResult>('/api/favorites')
}

function readLocalFavorites(): FavoriteItem[] {
  try {
    const rawValue = localStorage.getItem(LOCAL_FAVORITES_KEY)
    if (!rawValue) return []
    const value: unknown = JSON.parse(rawValue)
    if (!isRecord(value) || !isRecord(value.state) || !Array.isArray(value.state.items)) {
      return []
    }
    return value.state.items.filter(isFavoriteItem)
  } catch {
    return []
  }
}

function isFavoriteItem(value: unknown): value is FavoriteItem {
  return (
    isRecord(value) &&
    typeof value.id === 'string' &&
    typeof value.title === 'string' &&
    typeof value.author === 'string' &&
    typeof value.description === 'string' &&
    typeof value.image === 'string' &&
    Array.isArray(value.tags) &&
    value.tags.every(tag => typeof tag === 'string') &&
    typeof value.favoritedAt === 'number'
  )
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}
