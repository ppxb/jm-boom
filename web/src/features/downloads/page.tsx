import { FolderOpenIcon, LoaderCircleIcon } from 'lucide-react'

import { BackTopButton } from '@/components/back-top-button'
import { EmptyState } from '@/components/empty-state'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
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
    removeTask,
    openTaskDir,
    openRootDir
  } = useDownloadTasks()

  return (
    <main className="relative min-h-screen bg-background p-[32px_32px_16px_96px] text-foreground">
      <div className="mx-auto max-w-6xl space-y-6">
        <PageHeader title="下载" description="查看下载进度、剩余时间和已完成文件目录">
          <Button
            variant="outline"
            size="sm"
            disabled={openRootDir.isPending}
            onClick={() => openRootDir.mutate()}
          >
            <FolderOpenIcon className="size-4" />
            下载目录
          </Button>
        </PageHeader>

        <Tabs value={filter} onValueChange={value => setFilter(value as DownloadFilter)}>
          <TabsList>
            {DOWNLOAD_FILTERS.map(item => (
              <TabsTrigger key={item.value} value={item.value} className="min-w-20">
                {item.label}
                <span className="ml-1 text-muted-foreground tabular-nums">
                  {filterCounts[item.value]}
                </span>
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>

        {tasks.isLoading ? (
          <div className="flex h-64 items-center justify-center text-sm text-muted-foreground">
            <LoaderCircleIcon className="mr-2 size-4 animate-spin" />
            正在读取下载任务
          </div>
        ) : tasks.isError ? (
          <Card>
            <CardContent className="p-6 text-sm text-destructive">
              {tasks.error.message}
            </CardContent>
          </Card>
        ) : taskList.length === 0 ? (
          <EmptyState emoji="(˘･_･˘)" title="暂无下载任务" />
        ) : filteredTasks.length === 0 ? (
          <EmptyState emoji="(˘･_･˘)" title="当前筛选下暂无任务" />
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
                isOpening={openTaskDir.isPending}
                onCancel={() => cancelTask.mutate(task.taskId)}
                onPause={() => pauseTask.mutate(task.taskId)}
                onResume={() => resumeTask.mutate(task.taskId)}
                onRemove={() => removeTask.mutate(task.taskId)}
                onOpen={() => openTaskDir.mutate(task.taskId)}
              />
            ))}
          </div>
        )}
      </div>
      <BackTopButton />
    </main>
  )
}
