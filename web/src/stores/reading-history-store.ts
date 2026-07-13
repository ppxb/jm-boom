import { create } from 'zustand'
import { persist } from 'zustand/middleware'

import { HISTORY } from '@/lib/constants'

export type ReadingHistoryItem = {
  comicId: string
  albumId: string
  title: string
  author: string
  coverUrl: string
  chapterId: string
  chapterTitle: string
  pageIndex: number
  pageCount: number
  updatedAt: number
}

type ReadingHistoryState = {
  items: ReadingHistoryItem[]
  upsert: (item: Omit<ReadingHistoryItem, 'updatedAt'>) => void
  remove: (comicId: string) => void
  removeMany: (comicIds: string[]) => void
  clear: () => void
}

export const useReadingHistoryStore = create<ReadingHistoryState>()(
  persist(
    set => ({
      items: [],
      upsert: item => {
        const nextItem: ReadingHistoryItem = {
          ...item,
          updatedAt: Date.now()
        }

        set(state => {
          const items = [
            nextItem,
            ...state.items.filter(entry => entry.comicId !== item.comicId)
          ].slice(0, HISTORY.MAX_ITEMS)

          return { items }
        })
      },
      remove: comicId => {
        set(state => ({
          items: state.items.filter(item => item.comicId !== comicId)
        }))
      },
      removeMany: comicIds => {
        const comicIdSet = new Set(comicIds)

        set(state => ({
          items: state.items.filter(item => !comicIdSet.has(item.comicId))
        }))
      },
      clear: () => {
        set({ items: [] })
      }
    }),
    {
      name: 'jm-boom-reading-history',
      version: 1,
      partialize: state => ({ items: state.items }),
      migrate: persistedState => ({ items: persistedHistoryItems(persistedState) }),
      merge: (persistedState, currentState) => ({
        ...currentState,
        items: persistedHistoryItems(persistedState)
      })
    }
  )
)

function persistedHistoryItems(persistedState: unknown) {
  if (
    typeof persistedState !== 'object' ||
    persistedState === null ||
    !('items' in persistedState) ||
    !Array.isArray(persistedState.items)
  ) {
    return []
  }

  return (persistedState.items as ReadingHistoryItem[]).slice(0, HISTORY.MAX_ITEMS)
}
