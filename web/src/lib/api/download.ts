import { apiClient } from './client'

export type DownloadChapterRequest = {
  chapterId: string
  title: string
  order: number
}

export type EnqueueDownloadRequest = {
  albumId: string
  comicTitle: string
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
  chapters: DownloadChapterRequest[]
  status: DownloadTaskStatus
  currentChapterTitle: string
  totalPages: number
  completedPages: number
  etaSeconds: number | null
  speedBytesPerSecond: number
  error: string | null
  createdAt: number
  startedAt: number | null
  updatedAt: number
  completedAt: number | null
}

export type DownloadTaskListResult = {
  tasks: DownloadTask[]
}

export async function enqueueComicDownload(
  request: EnqueueDownloadRequest
): Promise<DownloadTaskListResult> {
  return apiClient.post('/api/downloads', request)
}

export async function listDownloadTasks(): Promise<DownloadTaskListResult> {
  return apiClient.get('/api/downloads')
}

export async function cancelDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return apiClient.post(`/api/downloads/${taskId}/cancel`)
}

export async function pauseDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return apiClient.post(`/api/downloads/${taskId}/pause`)
}

export async function resumeDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return apiClient.post(`/api/downloads/${taskId}/resume`)
}

export async function removeDownloadTask(taskId: string): Promise<DownloadTaskListResult> {
  return apiClient.delete(`/api/downloads/${taskId}`)
}
