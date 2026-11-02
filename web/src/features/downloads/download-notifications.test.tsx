import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import {
  defaultScheduler,
  notifyManager,
  QueryClient,
  QueryClientProvider
} from '@tanstack/react-query'

import type { DownloadTask } from '@/lib/api/download'
import { queryKeys } from '@/lib/query-keys'
import { DownloadNotifications, markDownloadSubmitted } from './download-notifications'

const notices = vi.hoisted(() => ({
  info: vi.fn(),
  success: vi.fn(),
  error: vi.fn(),
  dismiss: vi.fn()
}))
vi.mock('sonner', () => ({ toast: notices }))

const task: DownloadTask = {
  taskId: 'task-1',
  albumId: '100',
  comicTitle: '测试漫画',
  chapters: [],
  status: 'queued',
  currentChapterTitle: '',
  totalPages: 10,
  completedPages: 0,
  etaSeconds: null,
  speedBytesPerSecond: 0,
  error: null,
  createdAt: 1,
  startedAt: null,
  updatedAt: 1,
  completedAt: null
}
let root: Root
let client: QueryClient

beforeEach(() => {
  vi.clearAllMocks()
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true)
  client = new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: Infinity } } })
  client.setQueryData(queryKeys.downloadTasks(), { tasks: [] })
  notifyManager.setScheduler(queueMicrotask)
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
})

afterEach(async () => {
  await act(async () => root.unmount())
  client.clear()
  document.body.replaceChildren()
  notifyManager.setScheduler(defaultScheduler)
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

async function render(page = '下载页') {
  await act(async () =>
    root.render(
      <QueryClientProvider client={client}>
        <DownloadNotifications />
        <p>{page}</p>
      </QueryClientProvider>
    )
  )
}

async function update(tasks: DownloadTask[]) {
  await act(async () => {
    client.setQueryData(queryKeys.downloadTasks(), { tasks })
  })
}

test('keeps one progress toast across page changes and announces completion once', async () => {
  await render()
  await update([task])
  expect(notices.info).toHaveBeenCalledTimes(1)
  expect(notices.info.mock.calls[0][1]).toMatchObject({ id: 'download:task-1', duration: Infinity })
  await render('探索页')
  await update([{ ...task, status: 'running', completedPages: 5, updatedAt: 2 }])
  expect(notices.info).toHaveBeenCalledTimes(2)
  expect(notices.info.mock.calls[1][1].id).toBe('download:task-1')
  const completed = { ...task, status: 'completed' as const, completedPages: 10, updatedAt: 3 }
  await update([completed])
  expect(notices.success).toHaveBeenCalledWith(
    '测试漫画',
    expect.objectContaining({ id: 'download:task-1', description: '下载完成', icon: undefined })
  )
  await render('设置页')
  await update([{ ...completed }])
  expect(notices.success).toHaveBeenCalledTimes(1)
})

test('does not replay historical results and respects dismissal until a task finishes', async () => {
  client.setQueryData(queryKeys.downloadTasks(), { tasks: [{ ...task, status: 'completed' }] })
  await render()
  expect(notices.success).not.toHaveBeenCalled()
  await update([{ ...task, status: 'running', updatedAt: 2 }])
  notices.info.mock.calls[0][1].onDismiss()
  await update([{ ...task, status: 'running', completedPages: 1, updatedAt: 3 }])
  expect(notices.info).toHaveBeenCalledTimes(1)
  await update([{ ...task, status: 'failed', error: '图片下载失败', updatedAt: 4 }])
  expect(notices.error).toHaveBeenCalledWith(
    '测试漫画',
    expect.objectContaining({ description: '图片下载失败' })
  )
  await update([{ ...task, updatedAt: 5 }])
  expect(notices.info).toHaveBeenCalledTimes(2)
  await update([])
  expect(notices.dismiss).toHaveBeenCalledWith('download:task-1')
})

test('shows a reconnecting state without falsely completing a running task', async () => {
  client.setQueryData(queryKeys.downloadTasks(), { tasks: [{ ...task, status: 'running' }] })
  await render()
  const fetch = vi.spyOn(globalThis, 'fetch').mockRejectedValue(new Error('offline'))
  await act(async () => {
    await client.refetchQueries({ queryKey: queryKeys.downloadTasks() })
  })
  expect(notices.info.mock.lastCall?.[1].description).toBe('正在重新连接下载服务…')
  expect(notices.success).not.toHaveBeenCalled()
  fetch.mockResolvedValue(
    new Response(JSON.stringify({ tasks: [{ ...task, status: 'completed', completedPages: 10 }] }))
  )
  await act(async () => {
    await client.refetchQueries({ queryKey: queryKeys.downloadTasks() })
  })
  expect(notices.success).toHaveBeenCalledTimes(1)
})

test('removes task toasts when leaving the authenticated app and can resume tracking', async () => {
  client.setQueryData(queryKeys.downloadTasks(), { tasks: [{ ...task, status: 'running' }] })
  await render()
  expect(notices.info).toHaveBeenCalledTimes(1)
  await act(async () => root.render(<p>登录页</p>))
  expect(notices.dismiss).toHaveBeenCalledWith('download:task-1')
  await render()
  expect(notices.info).toHaveBeenCalledTimes(2)
})

test('announces a locally submitted task that is already finished in the first snapshot', async () => {
  markDownloadSubmitted(task.taskId)
  client.setQueryData(queryKeys.downloadTasks(), {
    taskId: task.taskId,
    tasks: [{ ...task, status: 'completed' }]
  })
  await render()
  expect(notices.success).toHaveBeenCalledTimes(1)
  await render('探索页')
  expect(notices.success).toHaveBeenCalledTimes(1)
})

test('does not lose submission feedback when the initial GET already saw its result', async () => {
  const completed = { ...task, status: 'completed' as const }
  client.setQueryData(queryKeys.downloadTasks(), { tasks: [completed] })
  await render()
  expect(notices.success).not.toHaveBeenCalled()
  markDownloadSubmitted(task.taskId)
  await act(async () => {
    client.setQueryData(queryKeys.downloadTasks(), { taskId: task.taskId, tasks: [completed] })
  })
  expect(notices.success).toHaveBeenCalledTimes(1)
})
