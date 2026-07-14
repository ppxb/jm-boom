import { create } from 'zustand'
import { persist } from 'zustand/middleware'

import type { ComicSummary } from '@/domain/comic'
import { HISTORY } from '@/lib/constants'

type ReadingHistoryComic = Pick<ComicSummary, 'id' | 'title' | 'author' | 'image'>

export type ReadingHistoryItem = ReadingHistoryComic & {
  chapterId: string
  chapterTitle: string
  pageIndex: number
  pageCount: number
  lastReadAt: number
}

type ReadingHistoryState = {
  items: ReadingHistoryItem[]
  upsert: (item: Omit<ReadingHistoryItem, 'lastReadAt'>) => void
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
          lastReadAt: Date.now()
        }

        set(state => {
          const items = [nextItem, ...state.items.filter(entry => entry.id !== item.id)].slice(
            0,
            HISTORY.MAX_ITEMS
          )

          return { items }
        })
      },
      remove: comicId => {
        set(state => ({
          items: state.items.filter(item => item.id !== comicId)
        }))
      },
      removeMany: comicIds => {
        const comicIdSet = new Set(comicIds)

        set(state => ({
          items: state.items.filter(item => !comicIdSet.has(item.id))
        }))
      },
      clear: () => {
        set({ items: [] })
      }
    }),
    {
      name: 'jm-boom-reading-history-v2'
    }
  )
)
