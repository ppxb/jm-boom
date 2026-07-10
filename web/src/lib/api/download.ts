import { tauriInvoke } from './tauri'

export type DownloadChapterRequest = {
  chapterId: string
  title: string
  order: number
}

export type EnqueueDownloadRequest = {
  albumId: string
  comicTitle: string
  endpoint?: string | null
  chapters: DownloadChapterRequest[]
}

export type DownloadTaskStatus =
  | 'queued'
  | 'running'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled'

export type DownloadTask = {
  taskId: string
  albumId: string
  comicTitle: string
  endpoint: string
  chapters: DownloadChapterRequest[]
  status: DownloadTaskStatus
  currentChapterTitle: string
  totalPages: number
  completedPages: number
  etaSeconds: number | null
  speedBytesPerSecond: number
  outputDir: string
  error: string | null
  createdAt: number
  startedAt: number | null
  updatedAt: number
  completedAt: number | null
}

export type DownloadTaskListResult = {
  rootDir: string
  tasks: DownloadTask[]
}

export async function enqueueComicDownload(
  request: EnqueueDownloadRequest
): Promise<DownloadTaskListResult> {
  return tauriInvoke<DownloadTaskListResult>('enqueue_comic_download', { request })
}

export async function listDownloadTasks(): Promise<DownloadTaskListResult> {
  return tauriInvoke<DownloadTaskListResult>('list_download_tasks')
}

export async function cancelDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return tauriInvoke<DownloadTaskListResult>('cancel_download_task', { taskId })
}

export async function pauseDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return tauriInvoke<DownloadTaskListResult>('pause_download_task', { taskId })
}

export async function resumeDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return tauriInvoke<DownloadTaskListResult>('resume_download_task', { taskId })
}

export async function removeDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return tauriInvoke<DownloadTaskListResult>('remove_download_task', { taskId })
}

export async function openDownloadTaskDir(taskId: string): Promise<void> {
  return tauriInvoke<void>('open_download_task_dir', { taskId })
}

export async function openDownloadRootDir(): Promise<void> {
  return tauriInvoke<void>('open_download_root_dir')
}
