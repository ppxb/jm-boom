import { afterEach, beforeEach, describe, expect, vi, test } from 'vitest'
import {
  defaultScheduler,
  notifyManager,
  QueryClient,
  QueryClientProvider
} from '@tanstack/react-query'
import { Window } from 'happy-dom'
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import type { ComicDetail } from '@/domain/comic'
import { useComicDownload } from '@/features/comic-detail/use-comic-download'
import type { DownloadTask, DownloadTaskListResult } from '@/lib/api/download'
import { queryKeys } from '@/lib/query-keys'
import { useDownloadTasks } from './use-download-tasks'

type Request = {
  path: string
  method: string
  signal: AbortSignal | null | undefined
  finish: (response: Response) => void
}

const tasksKey = queryKeys.downloadTasks()
const chaptersKey = queryKeys.downloadedChapters()
const emptyList: DownloadTaskListResult = { tasks: [] }
const comic: ComicDetail = {
  id: '100',
  title: 'Test comic',
  description: '',
  image: '',
  tags: [],
  authors: [],
  actors: [],
  works: [],
  totalViews: 0,
  likes: 0,
  commentCount: 0,
  totalPhotos: 0,
  relatedComics: [],
  chapters: [{ id: '101', title: 'Chapter 1', sort: '1' }]
}

let dom: Window
let root: Root
let queryClient: QueryClient
let globals: Map<string, PropertyDescriptor | undefined>
let requests: Request[]
let downloads: ReturnType<typeof useDownloadTasks>
let comicDownload: ReturnType<typeof useComicDownload>

function task(taskId = 'a', status: DownloadTask['status'] = 'completed'): DownloadTask {
  return {
    taskId,
    albumId: comic.id,
    comicTitle: comic.title,
    chapters: [{ chapterId: '101', title: 'Chapter 1', order: 1 }],
    status,
    currentChapterTitle: 'Chapter 1',
    totalPages: 10,
    completedPages: status === 'completed' ? 10 : 3,
    etaSeconds: null,
    speedBytesPerSecond: 0,
    error: null,
    createdAt: 1,
    startedAt: 1,
    updatedAt: 1,
    completedAt: status === 'completed' ? 2 : null
  }
}

beforeEach(() => {
  dom = new Window({ url: 'http://localhost' })
  const values = {
    window: dom,
    document: dom.document,
    navigator: dom.navigator,
    IS_REACT_ACT_ENVIRONMENT: true
  }
  globals = new Map(
    Object.keys(values).map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)])
  )
  for (const [key, value] of Object.entries(values)) {
    Object.defineProperty(globalThis, key, { value, configurable: true, writable: true })
  }
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
  queryClient = new QueryClient({ defaultOptions: { queries: { gcTime: Infinity, retry: false } } })
  queryClient.setQueryData(tasksKey, { tasks: [task()] })
  queryClient.setQueryData(chaptersKey, { chapterIds: ['101'] })
  notifyManager.setScheduler(queueMicrotask)
  requests = []
  const mockFetch = (input: Parameters<typeof fetch>[0], init?: Parameters<typeof fetch>[1]) =>
    new Promise<Response>(finish => {
      // Deliberately allow aborted responses to arrive, exercising Query's cancellation guard.
      requests.push({
        path: String(input),
        method: init?.method ?? 'GET',
        signal: init?.signal,
        finish
      })
    })
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => {
    root.unmount()
    for (const request of requests) request.finish(unavailable())
  })
  queryClient.clear()
  await dom.happyDOM.close()
  notifyManager.setScheduler(defaultScheduler)
  vi.restoreAllMocks()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

function Tasks() {
  downloads = useDownloadTasks()
  return (
    <output>{downloads.taskList.map(task => `${task.taskId}:${task.status}`).join(',')}</output>
  )
}

function ComicDownload() {
  comicDownload = useComicDownload(comic, comic.chapters)
  return null
}

async function render(showTasks = true) {
  await act(async () => {
    root.render(
      <QueryClientProvider client={queryClient}>
        {showTasks ? <Tasks /> : null}
        <ComicDownload />
      </QueryClientProvider>
    )
  })
}

function listRequests() {
  return requests.filter(request => request.path === '/api/downloads' && request.method === 'GET')
}

function latestListRequest() {
  const list = listRequests()
  return list[list.length - 1]
}

function writeRequest(path: string) {
  const request = requests.find(request => request.path === path && request.method !== 'GET')
  expect(request).toBeDefined()
  return request!
}

async function beginListRequest() {
  await act(async () => {
    void downloads.tasks.refetch()
  })
  return latestListRequest()
}

async function respond(request: Request, result: DownloadTaskListResult) {
  await act(async () => request.finish(Response.json(result)))
}

function unavailable() {
  return Response.json({ error: 'Unavailable', retryable: false }, { status: 503 })
}

function cachedTasks() {
  return queryClient.getQueryData<DownloadTaskListResult>(tasksKey)
}

describe('download mutation cache synchronization', () => {
  test('does not restore a deleted completed task when an older list response arrives', async () => {
    await render()
    const oldList = await beginListRequest()
    await act(async () => downloads.removeTasks.mutate(['a']))
    await respond(writeRequest('/api/downloads/a'), emptyList)
    expect(cachedTasks()).toEqual(emptyList)
    await respond(oldList, { tasks: [task()] })
    expect(cachedTasks()).toEqual(emptyList)
    expect(document.body.textContent).not.toContain('a:completed')
    expect(oldList.signal?.aborted).toBe(true)
    expect(listRequests()).toHaveLength(2)
    await respond(latestListRequest(), emptyList)
    expect(downloads.removeTasks.isSuccess).toBe(true)
    expect(queryClient.getQueryState(chaptersKey)?.isInvalidated).toBe(true)
  })

  test.each([
    { operation: 'pause', mutation: 'pauseTask', before: 'running', after: 'paused' },
    { operation: 'resume', mutation: 'resumeTask', before: 'paused', after: 'queued' },
    { operation: 'cancel', mutation: 'cancelTask', before: 'running', after: 'cancelled' }
  ] as const)(
    'protects $operation from a list request started during the operation',
    async ({ operation, mutation, before, after }) => {
      const oldSnapshot = { tasks: [task('a', before)] }
      const newSnapshot = { tasks: [task('a', after)] }
      queryClient.setQueryData(tasksKey, oldSnapshot)
      await render()
      await act(async () => downloads[mutation].mutate('a'))
      const duringWrite = await beginListRequest()
      await respond(writeRequest(`/api/downloads/a/${operation}`), newSnapshot)
      await respond(duringWrite, oldSnapshot)
      expect(cachedTasks()).toEqual(newSnapshot)
      expect(duringWrite.signal?.aborted).toBe(true)
      expect(listRequests()).toHaveLength(2)
      await respond(latestListRequest(), newSnapshot)
      expect(downloads[mutation].isSuccess).toBe(true)
    }
  )

  test('does not hide a newly enqueued task behind an old empty list response', async () => {
    queryClient.setQueryData(tasksKey, emptyList)
    await render()
    const oldList = await beginListRequest()
    await act(async () => comicDownload.start())
    const enqueued = { taskId: 'new', tasks: [task('new', 'queued')] }
    await respond(writeRequest('/api/downloads'), enqueued)
    await respond(oldList, emptyList)
    expect(cachedTasks()).toEqual(enqueued)
    expect(oldList.signal?.aborted).toBe(true)
    expect(listRequests()).toHaveLength(2)
    const running = { tasks: [task('new', 'running')] }
    await respond(latestListRequest(), running)
    expect(cachedTasks()).toEqual(running)
    expect(comicDownload.isPending).toBe(false)
  })

  test('keeps a successful mutation snapshot when the final list refresh fails', async () => {
    await render()
    await act(async () => downloads.removeTasks.mutate(['a']))
    await respond(writeRequest('/api/downloads/a'), emptyList)
    expect(listRequests()).toHaveLength(1)
    await act(async () => latestListRequest().finish(unavailable()))
    expect(cachedTasks()).toEqual(emptyList)
    expect(downloads.removeTasks.isSuccess).toBe(true)
    expect(downloads.tasks.isError).toBe(true)
  })

  test('reconciles partially completed batch deletion even when a later delete fails', async () => {
    const original = { tasks: [task('a'), task('b')] }
    const remaining = { tasks: [task('b')] }
    queryClient.setQueryData(tasksKey, original)
    await render()
    await act(async () => downloads.removeTasks.mutate(['a', 'b']))
    const duringWrite = await beginListRequest()
    await respond(writeRequest('/api/downloads/a'), remaining)
    await act(async () => writeRequest('/api/downloads/b').finish(unavailable()))
    expect(duringWrite.signal?.aborted).toBe(true)
    expect(listRequests()).toHaveLength(2)
    await respond(latestListRequest(), remaining)
    await respond(duringWrite, original)
    expect(cachedTasks()).toEqual(remaining)
    expect(downloads.removeTasks.isError).toBe(true)
    expect(queryClient.getQueryState(chaptersKey)?.isInvalidated).toBe(true)
  })

  test('restarts reconciliation after a failed mutation even when the list has no cached data', async () => {
    queryClient.removeQueries({ queryKey: tasksKey })
    await render()
    const firstLoad = latestListRequest()
    await act(async () => downloads.pauseTask.mutate('a'))
    const duringWrite = await beginListRequest()
    await act(async () => writeRequest('/api/downloads/a/pause').finish(unavailable()))
    expect(firstLoad.signal?.aborted).toBe(true)
    expect(duringWrite.signal?.aborted).toBe(true)
    expect(listRequests()).toHaveLength(3)
    const actual = { tasks: [task('a', 'paused')] }
    await respond(latestListRequest(), actual)
    await respond(firstLoad, emptyList)
    await respond(duringWrite, emptyList)
    expect(cachedTasks()).toEqual(actual)
    expect(downloads.pauseTask.isError).toBe(true)
  })

  test('marks an inactive download list stale after enqueue so entering downloads refreshes it', async () => {
    await render(false)
    await act(async () => comicDownload.start())
    const enqueued = { taskId: 'new', tasks: [task('new', 'queued')] }
    await respond(writeRequest('/api/downloads'), enqueued)
    expect(cachedTasks()).toEqual(enqueued)
    expect(queryClient.getQueryState(tasksKey)?.isInvalidated).toBe(true)
    expect(listRequests()).toHaveLength(0)
    await render()
    expect(listRequests()).toHaveLength(1)
    const running = { tasks: [task('new', 'running')] }
    await respond(latestListRequest(), running)
    expect(cachedTasks()).toEqual(running)
  })

  test('reconciles overlapping mutations whose response snapshots arrive in reverse order', async () => {
    queryClient.setQueryData(tasksKey, { tasks: [task('a', 'running'), task('b', 'running')] })
    await render()
    await act(async () => {
      downloads.pauseTask.mutate('a')
      downloads.cancelTask.mutate('b')
    })
    const actual = { tasks: [task('a', 'paused'), task('b', 'cancelled')] }
    await respond(writeRequest('/api/downloads/b/cancel'), actual)
    expect(listRequests()).toHaveLength(1)
    await respond(latestListRequest(), actual)
    await respond(writeRequest('/api/downloads/a/pause'), {
      tasks: [task('a', 'paused'), task('b', 'running')]
    })
    expect(listRequests()).toHaveLength(2)
    await respond(latestListRequest(), actual)
    expect(cachedTasks()).toEqual(actual)
    expect(downloads.pauseTask.isSuccess).toBe(true)
    expect(downloads.cancelTask.isSuccess).toBe(true)
  })
})
