import { LoaderCircleIcon, Trash2Icon } from 'lucide-react'

import { AppPage } from '@/components/app-page'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { SelectionActions } from '@/components/selection-actions'
import { Card, CardContent } from '@/components/ui/card'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { DownloadTaskCard } from './download-task-card'
import { useDownloadSelection } from './use-download-selection'
import { DOWNLOAD_FILTERS, type DownloadFilter, useDownloadTasks } from './use-download-tasks'

export function DownloadsPage() {
  const {
    filter,
    setFilter,
    tasks,
    taskList,
    filteredTasks,
    filterCounts,
    cancelTask,
    pauseTask,
    resumeTask,
    removeTasks
  } = useDownloadTasks()
  const selection = useDownloadSelection(filteredTasks)

  function removeSelectedTasks() {
    const taskIds = [...selection.selectedTaskIds]
    if (taskIds.length === 0) return
    removeTasks.mutate(taskIds, {
      onSuccess: () => selection.toggleSelectionMode(false)
    })
  }

  return (
    <AppPage>
      <PageHeader title="下载" description="可查看和管理下载任务">
        <SelectionActions
          isSelecting={selection.isSelecting}
          allSelected={selection.allSelected}
          selectedCount={selection.selectedCount}
          disabled={filteredTasks.length === 0}
          loading={removeTasks.isPending}
          enterLabel="删除"
          enterIcon={<Trash2Icon className="size-4" />}
          dialogTitle="删除下载任务"
          dialogDescription={`将删除选中的 ${selection.selectedCount} 个下载任务和离线文件，此操作不可撤销。`}
          onEnter={() => selection.toggleSelectionMode(true)}
          onExit={() => selection.toggleSelectionMode(false)}
          onToggleAll={selection.toggleSelectAll}
          onDeleteSelected={removeSelectedTasks}
        />
      </PageHeader>

      <Tabs
        value={filter}
        className="w-full"
        onValueChange={value => setFilter(value as DownloadFilter)}
      >
        <TabsList className="grid w-full grid-cols-4">
          {DOWNLOAD_FILTERS.map(item => (
            <TabsTrigger key={item.value} value={item.value}>
              <span className="truncate">{item.label}</span>
              <span className="shrink-0 text-muted-foreground tabular-nums">
                {filterCounts[item.value]}
              </span>
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>

      {tasks.isLoading ? (
        <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
          <LoaderCircleIcon className="mr-2 size-4 animate-spin" />
          正在读取下载任务
        </div>
      ) : tasks.isError ? (
        <Card>
          <CardContent className="p-6 text-sm text-destructive">{tasks.error.message}</CardContent>
        </Card>
      ) : taskList.length === 0 ? (
        <EmptyState className="min-h-0 flex-1" emoji="(˘･_･˘)" title="暂无下载任务" />
      ) : filteredTasks.length === 0 ? (
        <EmptyState className="min-h-0 flex-1" emoji="(˘･_･˘)" title="当前筛选下暂无任务" />
      ) : (
        <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4 lg:grid-cols-4 lg:gap-6">
          {filteredTasks.map(task => (
            <DownloadTaskCard
              key={task.taskId}
              task={task}
              isCancelling={cancelTask.isPending && cancelTask.variables === task.taskId}
              isPausing={pauseTask.isPending && pauseTask.variables === task.taskId}
              isResuming={resumeTask.isPending && resumeTask.variables === task.taskId}
              selectable={selection.isSelecting}
              selected={selection.selectedTaskIds.has(task.taskId)}
              onCancel={() => cancelTask.mutate(task.taskId)}
              onPause={() => pauseTask.mutate(task.taskId)}
              onResume={() => resumeTask.mutate(task.taskId)}
              onSelect={selection.toggleSelectItem}
            />
          ))}
        </div>
      )}
    </AppPage>
  )
}
