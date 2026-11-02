import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import type { DownloadTask } from '@/lib/api/download'
import { DownloadTaskCard } from './download-task-card'

const { navigate, notices } = vi.hoisted(() => ({
  navigate: vi.fn(),
  notices: { info: vi.fn(), success: vi.fn(), error: vi.fn(), dismiss: vi.fn() }
}))
vi.mock('@tanstack/react-router', async importOriginal => ({
  ...(await importOriginal<typeof import('@tanstack/react-router')>()),
  useNavigate: () => navigate
}))
vi.mock('sonner', () => ({ toast: notices }))

const task: DownloadTask = {
  taskId: 'task-1',
  albumId: '100',
  comicTitle: '测试漫画',
  comicAuthor: '测试作者',
  chapters: [{ chapterId: '101', title: '正文', order: 1 }],
  status: 'completed',
  currentChapterTitle: '正文',
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
let root: Root
let downloads: string[]

beforeEach(() => {
  vi.clearAllMocks()
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true)
  vi.stubGlobal(
    'URL',
    class extends URL {
      static createObjectURL = vi.fn(() => 'blob:download-test')
      static revokeObjectURL = vi.fn()
    }
  )
  downloads = []
  vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (
    this: HTMLAnchorElement
  ) {
    downloads.push(this.download)
  })
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
})

afterEach(async () => {
  await act(async () => root.unmount())
  document.body.replaceChildren()
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

async function render(overrides: Partial<DownloadTask> = {}, selectable = false) {
  await act(async () =>
    root.render(
      <DownloadTaskCard
        task={{ ...task, ...overrides }}
        isCancelling={false}
        isPausing={false}
        isResuming={false}
        selectable={selectable}
        selected={false}
        onCancel={vi.fn()}
        onPause={vi.fn()}
        onResume={vi.fn()}
        onSelect={vi.fn()}
      />
    )
  )
}

async function exportMenuItem(label: string) {
  await act(async () =>
    document.querySelector<HTMLButtonElement>('[aria-label="导出下载"]')!.click()
  )
  const item = Array.from(document.querySelectorAll<HTMLElement>('[role="menuitem"]')).find(item =>
    item.textContent?.includes(label)
  )!
  expect(item).toBeDefined()
  await act(async () => item.click())
}

test('shows author and chapter count on separate lines and hides export while selecting or unfinished', async () => {
  await render()
  expect(Array.from(document.querySelectorAll('p')).map(item => item.textContent)).toEqual([
    '测试作者',
    '1 个章节'
  ])
  expect(document.querySelector('[aria-label="导出下载"]')).not.toBeNull()
  await render({ comicAuthor: undefined })
  expect(document.querySelector('p')?.textContent).toBe('N/A')
  await render({}, true)
  expect(document.querySelector('[aria-label="导出下载"]')).toBeNull()
  await render({ status: 'running' })
  expect(document.querySelector('[aria-label="导出下载"]')).toBeNull()
})

test.each(['PDF', 'CBZ'])('exports %s without opening the reader', async format => {
  const filename = `JM100_测试漫画_测试作者.${format.toLowerCase()}`
  const fetch = vi.spyOn(globalThis, 'fetch').mockResolvedValue(
    new Response('export', {
      headers: {
        'Content-Type': format === 'PDF' ? 'application/pdf' : 'application/vnd.comicbook+zip',
        'Content-Disposition': `attachment; filename="download.${format.toLowerCase()}"; filename*=UTF-8''${encodeURIComponent(filename)}`
      }
    })
  )
  await render()
  await exportMenuItem(`导出 ${format}`)
  expect(fetch.mock.calls[0][0]).toBe(`/api/downloads/task-1/export?format=${format.toLowerCase()}`)
  expect(downloads).toEqual([filename])
  expect(navigate).not.toHaveBeenCalled()
  expect(notices.success).toHaveBeenCalled()
})

test('reports an export error without downloading the JSON error response', async () => {
  vi.spyOn(globalThis, 'fetch').mockResolvedValue(
    new Response(JSON.stringify({ error: '离线文件缺失' }), {
      status: 409,
      headers: { 'Content-Type': 'application/json' }
    })
  )
  await render()
  await exportMenuItem('导出 CBZ')
  expect(downloads).toEqual([])
  expect(notices.error).toHaveBeenCalledWith(
    '导出失败',
    expect.objectContaining({ id: 'download-export:task-1', description: '离线文件缺失' })
  )
  expect(navigate).not.toHaveBeenCalled()
})

test('export survives leaving the download page and uses the server filename', async () => {
  let finish!: (response: Response) => void
  let signal: AbortSignal | null | undefined
  vi.spyOn(globalThis, 'fetch').mockImplementation((_input, options) => {
    signal = options?.signal
    return new Promise(resolve => {
      finish = resolve
    })
  })
  await render()
  await exportMenuItem('导出 CBZ')
  await act(async () => root.render(<p>其他页面</p>))
  expect(signal?.aborted).toBe(false)
  const filename = 'JM100_测试漫画_测试作者.cbz'
  await act(async () =>
    finish(
      new Response('export', {
        headers: {
          'Content-Type': 'application/vnd.comicbook+zip',
          'Content-Disposition': `attachment; filename*=UTF-8''${encodeURIComponent(filename)}`
        }
      })
    )
  )
  expect(downloads).toEqual([filename])
  expect(notices.success).toHaveBeenCalledWith(
    expect.any(String),
    expect.objectContaining({ id: 'download-export:task-1' })
  )
})

test('the global export toast can cancel a request after its card unmounts', async () => {
  vi.spyOn(globalThis, 'fetch').mockImplementation(
    (_input, options) =>
      new Promise((_resolve, reject) => {
        options?.signal?.addEventListener('abort', () =>
          reject(new DOMException('Cancelled', 'AbortError'))
        )
      })
  )
  await render()
  await exportMenuItem('导出 PDF')
  const cancel = notices.info.mock.lastCall?.[1].action.onClick
  await act(async () => root.render(<p>其他页面</p>))
  const event = new Event('click', { cancelable: true })
  await act(async () => cancel(event))
  expect(event.defaultPrevented).toBe(true)
  expect(downloads).toEqual([])
  expect(notices.info).toHaveBeenCalledWith(
    '已取消导出',
    expect.objectContaining({ id: 'download-export:task-1', action: undefined })
  )
  await render()
  expect(document.querySelector<HTMLButtonElement>('[aria-label="导出下载"]')!.disabled).toBe(false)
})
