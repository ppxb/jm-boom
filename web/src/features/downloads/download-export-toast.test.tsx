import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import { Toaster, toast } from 'sonner'

import type { DownloadTask } from '@/lib/api/download'
import { startDownloadExport } from './download-export'

let root: Root
let pending: Promise<void> | undefined
let rejectRequest: ((error: Error) => void) | undefined

beforeEach(async () => {
  vi.useFakeTimers()
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true)
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
  await act(async () => root.render(<Toaster />))
  vi.spyOn(globalThis, 'fetch').mockImplementation(
    (_input, options) =>
      new Promise((_resolve, reject) => {
        rejectRequest = reject
        options?.signal?.addEventListener('abort', () =>
          reject(new DOMException('Cancelled', 'AbortError'))
        )
      })
  )
})

afterEach(async () => {
  await act(async () => {
    rejectRequest?.(new Error('test cleanup'))
    await pending
    toast.dismiss()
    root.unmount()
  })
  document.body.replaceChildren()
  vi.clearAllTimers()
  vi.useRealTimers()
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

test('the real Sonner action keeps the cancellation result visible after its close delay', async () => {
  const task: DownloadTask = {
    taskId: 'cancel-toast',
    albumId: '100',
    comicTitle: '测试漫画',
    comicAuthor: '',
    chapters: [],
    status: 'completed',
    currentChapterTitle: '',
    totalPages: 1,
    completedPages: 1,
    etaSeconds: null,
    speedBytesPerSecond: 0,
    error: null,
    createdAt: 1,
    startedAt: 1,
    updatedAt: 1,
    completedAt: 1
  }
  await act(async () => {
    pending = startDownloadExport(task, 'cbz')
    await vi.advanceTimersByTimeAsync(0)
  })
  const cancel = Array.from(document.querySelectorAll('button')).find(
    button => button.textContent === '取消导出'
  )
  expect(cancel).toBeDefined()
  await act(async () => {
    cancel!.click()
    await pending
    await vi.advanceTimersByTimeAsync(250)
  })
  const notification = document.querySelector('[data-sonner-toast]')
  expect(notification?.textContent).toContain('已取消导出')
  expect(notification?.getAttribute('data-removed')).not.toBe('true')
  expect(notification?.querySelector('[data-action]')).toBeNull()
})
