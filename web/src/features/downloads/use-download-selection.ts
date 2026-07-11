import { useEffect, useState } from 'react'

import type { DownloadTask } from '@/lib/api/download'

export function useDownloadSelection(tasks: DownloadTask[]) {
  const [isSelecting, setIsSelecting] = useState(false)
  const [selectedTaskIds, setSelectedTaskIds] = useState<Set<string>>(() => new Set())
  const selectedCount = selectedTaskIds.size
  const allSelected = tasks.length > 0 && selectedCount === tasks.length

  useEffect(() => {
    const availableTaskIds = new Set(tasks.map(task => task.taskId))
    setSelectedTaskIds(current => {
      const next = new Set([...current].filter(taskId => availableTaskIds.has(taskId)))
      return next.size === current.size ? current : next
    })
    if (tasks.length === 0) setIsSelecting(false)
  }, [tasks])

  function toggleSelectionMode(next: boolean) {
    setIsSelecting(next)
    if (!next) setSelectedTaskIds(new Set())
  }

  function toggleSelectAll() {
    setSelectedTaskIds(allSelected ? new Set() : new Set(tasks.map(task => task.taskId)))
  }

  function toggleSelectItem(taskId: string) {
    setSelectedTaskIds(current => {
      const next = new Set(current)
      if (next.has(taskId)) next.delete(taskId)
      else next.add(taskId)
      return next
    })
  }

  return {
    isSelecting,
    selectedTaskIds,
    selectedCount,
    allSelected,
    toggleSelectionMode,
    toggleSelectAll,
    toggleSelectItem
  }
}
