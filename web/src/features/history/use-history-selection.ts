import { useSelectionSet } from '@/lib/use-selection-set'

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
  const { selectedIds: selectedComicIds, ...selection } = useSelectionSet(items, getHistoryItemId)

  return {
    ...selection,
    selectedComicIds
  }
}

function getHistoryItemId(item: HistoryItem) {
  return item.id
}
