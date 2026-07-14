import { create } from 'zustand'
import { persist } from 'zustand/middleware'

import type { ComicSummary } from '@/domain/comic'

export type LocalFavoriteItem = ComicSummary & {
  favoritedAt: number
}

type LocalFavoritesState = {
  items: LocalFavoriteItem[]
  toggle: (comic: ComicSummary) => boolean
  remove: (comicId: string) => void
  clear: () => void
}

export const useLocalFavoritesStore = create<LocalFavoritesState>()(
  persist(
    set => ({
      items: [],
      toggle: comic => {
        let favorited = false

        set(state => {
          const exists = state.items.some(entry => entry.id === comic.id)
          favorited = !exists

          return {
            items: exists
              ? state.items.filter(entry => entry.id !== comic.id)
              : [{ ...comic, favoritedAt: Date.now() }, ...state.items]
          }
        })

        return favorited
      },
      remove: comicId => {
        set(state => ({ items: state.items.filter(item => item.id !== comicId) }))
      },
      clear: () => set({ items: [] })
    }),
    {
      name: 'jm-boom-local-favorites-v2'
    }
  )
)
