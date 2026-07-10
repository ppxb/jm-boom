import { create } from 'zustand'
import { persist } from 'zustand/middleware'

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
          const items = [nextItem, ...state.items.filter(entry => entry.comicId !== item.comicId)]

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
      name: 'jm-boom-reading-history'
    }
  )
)
