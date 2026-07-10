// import { apiClient } from './client'

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
  _request: EnqueueDownloadRequest
): Promise<DownloadTaskListResult> {
  // HTTP 模式不支持下载管理
  throw new Error('Download management not supported in HTTP mode')
}

export async function listDownloadTasks(): Promise<DownloadTaskListResult> {
  return {
    rootDir: '',
    tasks: []
  }
}

export async function cancelDownloadTask(_taskId: string): Promise<DownloadTaskListResult> {
  throw new Error('Download management not supported in HTTP mode')
}

export async function pauseDownloadTask(_taskId: string): Promise<DownloadTaskListResult> {
  throw new Error('Download management not supported in HTTP mode')
}

export async function resumeDownloadTask(_taskId: string): Promise<DownloadTaskListResult> {
  throw new Error('Download management not supported in HTTP mode')
}

export async function removeDownloadTask(_taskId: string): Promise<DownloadTaskListResult> {
  throw new Error('Download management not supported in HTTP mode')
}

export async function openDownloadTaskDir(_taskId: string): Promise<void> {
  throw new Error('Download management not supported in HTTP mode')
}

export async function openDownloadRootDir(): Promise<void> {
  throw new Error('Download management not supported in HTTP mode')
}
