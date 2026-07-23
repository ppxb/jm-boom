import { useEffect, useState } from 'react'

export function useSelectionSet<T>(items: readonly T[], getId: (item: T) => string) {
  const [isSelecting, setIsSelecting] = useState(false)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(() => new Set())
  const selectedCount = selectedIds.size
  const allSelected = items.length > 0 && selectedCount === items.length

  useEffect(() => {
    const availableIds = new Set(items.map(getId))

    setSelectedIds(current => {
      const next = new Set([...current].filter(id => availableIds.has(id)))
      return next.size === current.size ? current : next
    })

    if (items.length === 0) {
      setIsSelecting(false)
    }
  }, [getId, items])

  function toggleSelectionMode(nextSelecting: boolean) {
    setIsSelecting(nextSelecting)

    if (!nextSelecting) {
      setSelectedIds(new Set())
    }
  }

  function toggleSelectAll() {
    setSelectedIds(allSelected ? new Set() : new Set(items.map(getId)))
  }

  function toggleSelectItem(id: string) {
    setSelectedIds(current => {
      const next = new Set(current)

      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }

      return next
    })
  }

  return {
    isSelecting,
    selectedIds,
    selectedCount,
    allSelected,
    toggleSelectionMode,
    toggleSelectAll,
    toggleSelectItem
  }
}
