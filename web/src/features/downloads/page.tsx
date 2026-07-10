import { LoaderCircleIcon } from 'lucide-react'

import { AppPage } from '@/components/app-page'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { Card, CardContent } from '@/components/ui/card'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { DownloadTaskCard } from './download-task-card'
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
    removeTask
  } = useDownloadTasks()

  return (
    <AppPage>
      <PageHeader title="下载" description="可查看和管理下载任务" />

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
        <div className="space-y-3">
          {filteredTasks.map(task => (
            <DownloadTaskCard
              key={task.taskId}
              task={task}
              isCancelling={cancelTask.isPending}
              isPausing={pauseTask.isPending}
              isResuming={resumeTask.isPending}
              isRemoving={removeTask.isPending}
              onCancel={() => cancelTask.mutate(task.taskId)}
              onPause={() => pauseTask.mutate(task.taskId)}
              onResume={() => resumeTask.mutate(task.taskId)}
              onRemove={() => removeTask.mutate(task.taskId)}
            />
          ))}
        </div>
      )}
    </AppPage>
  )
}
