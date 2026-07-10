import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type LocalFavoriteItem = {
  id: string
  title: string
  author: string
  description: string
  image: string
  tags: string[]
  updatedAt: number
}

type LocalFavoritesState = {
  items: LocalFavoriteItem[]
  toggle: (item: Omit<LocalFavoriteItem, 'updatedAt'>) => boolean
  remove: (comicId: string) => void
  clear: () => void
}

export const useLocalFavoritesStore = create<LocalFavoritesState>()(
  persist(
    set => ({
      items: [],
      toggle: item => {
        let favorited = false

        set(state => {
          const exists = state.items.some(entry => entry.id === item.id)
          favorited = !exists

          return {
            items: exists
              ? state.items.filter(entry => entry.id !== item.id)
              : [{ ...item, updatedAt: Date.now() }, ...state.items]
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
      name: 'jm-boom-local-favorites'
    }
  )
)
