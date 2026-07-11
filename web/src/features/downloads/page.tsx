import { LoaderCircleIcon, Trash2Icon, XIcon } from 'lucide-react'

import { AppPage } from '@/components/app-page'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
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
        {selection.isSelecting ? (
          <>
            <Button
              type="button"
              variant="outline"
              size="sm"
              disabled={filteredTasks.length === 0 || removeTasks.isPending}
              onClick={selection.toggleSelectAll}
            >
              {selection.allSelected ? '取消全选' : '全选'}
            </Button>
            <ConfirmDialog
              trigger={
                <Button
                  type="button"
                  variant="destructive"
                  size="sm"
                  disabled={selection.selectedCount === 0 || removeTasks.isPending}
                >
                  <Trash2Icon className="size-4" />
                  删除选中
                </Button>
              }
              icon={<Trash2Icon className="size-5 text-destructive" />}
              title="删除下载任务"
              description={`将删除选中的 ${selection.selectedCount} 个下载任务和离线文件，此操作不可撤销。`}
              confirmText="确认删除"
              variant="destructive"
              loading={removeTasks.isPending}
              onConfirm={removeSelectedTasks}
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={removeTasks.isPending}
              onClick={() => selection.toggleSelectionMode(false)}
            >
              <XIcon className="size-4" />
              退出
            </Button>
          </>
        ) : (
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={filteredTasks.length === 0}
            onClick={() => selection.toggleSelectionMode(true)}
          >
            <Trash2Icon className="size-4" />
            删除
          </Button>
        )}
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
