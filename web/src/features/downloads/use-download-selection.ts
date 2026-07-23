import type { DownloadTask } from '@/lib/api/download'
import { useSelectionSet } from '@/lib/use-selection-set'

export function useDownloadSelection(tasks: DownloadTask[]) {
  const { selectedIds: selectedTaskIds, ...selection } = useSelectionSet(tasks, getDownloadTaskId)

  return {
    ...selection,
    selectedTaskIds
  }
}

function getDownloadTaskId(task: DownloadTask) {
  return task.taskId
}
