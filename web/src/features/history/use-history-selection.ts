import { useEffect, useState } from 'react'

export interface HistoryItem {
  id: string
  pageIndex: number
  pageCount: number
  lastReadAt: number
}

export interface HistorySelection {
  isSelecting: boolean
  selectedComicIds: Set<string>
  selectedCount: number
  allSelected: boolean
  toggleSelectionMode: (nextSelecting: boolean) => void
  toggleSelectAll: () => void
  toggleSelectItem: (comicId: string) => void
}

export function useHistorySelection(items: HistoryItem[]): HistorySelection {
  const [isSelecting, setIsSelecting] = useState(false)
  const [selectedComicIds, setSelectedComicIds] = useState<Set<string>>(() => new Set())

  const selectedCount = selectedComicIds.size
  const allSelected = items.length > 0 && selectedCount === items.length

  // Sync selection state when items change
  useEffect(() => {
    const availableComicIds = new Set(items.map(item => item.id))

    setSelectedComicIds(current => {
      const next = new Set([...current].filter(comicId => availableComicIds.has(comicId)))
      return next.size === current.size ? current : next
    })

    if (items.length === 0) {
      setIsSelecting(false)
    }
  }, [items])

  function toggleSelectionMode(nextSelecting: boolean) {
    setIsSelecting(nextSelecting)

    if (!nextSelecting) {
      setSelectedComicIds(new Set())
    }
  }

  function toggleSelectAll() {
    if (allSelected) {
      setSelectedComicIds(new Set())
    } else {
      setSelectedComicIds(new Set(items.map(item => item.id)))
    }
  }

  function toggleSelectItem(comicId: string) {
    setSelectedComicIds(current => {
      const next = new Set(current)

      if (next.has(comicId)) {
        next.delete(comicId)
      } else {
        next.add(comicId)
      }

      return next
    })
  }

  return {
    isSelecting,
    selectedComicIds,
    selectedCount,
    allSelected,
    toggleSelectionMode,
    toggleSelectAll,
    toggleSelectItem
  }
}
