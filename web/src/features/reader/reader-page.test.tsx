import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'
import {
  QueryClient,
  QueryClientProvider,
  notifyManager,
  defaultScheduler
} from '@tanstack/react-query'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { queryKeys } from '@/lib/query-keys'
import { useSettingsStore } from '@/stores/settings-store'
import { ReaderPage } from './reader-page'
import { useReaderHistorySync } from './use-reader-history-sync'

const { navigate } = vi.hoisted(() => ({ navigate: vi.fn(async () => {}) }))
vi.mock('@tanstack/react-router', async importOriginal => ({
  ...(await importOriginal<typeof import('@tanstack/react-router')>()),
  useNavigate: () => navigate,
  useRouter: () => ({ history: { back: vi.fn() } })
}))
vi.mock('./reader-bars', () => ({
  ReaderTopBar: () => null,
  ReaderBottomBar: ({
    currentReadId,
    currentIndex,
    pageCount
  }: {
    currentReadId: string
    currentIndex: number
    pageCount: number
  }) => (
    <output>
      {currentReadId}:{currentIndex}:{pageCount}
    </output>
  )
}))
vi.mock('./use-reader-history-sync', () => ({ useReaderHistorySync: vi.fn() }))
vi.mock('./use-reader-preload', () => ({ useReaderPreload: vi.fn() }))
vi.mock('./use-next-chapter-prefetch', () => ({ useNextChapterPrefetch: vi.fn() }))

let root: Root
let container: HTMLDivElement
let client: QueryClient

beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true)
  vi.useFakeTimers()
  notifyManager.setScheduler(queueMicrotask)
  container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
  client = new QueryClient({ defaultOptions: { queries: { gcTime: Infinity, retry: false } } })
  useSettingsStore.setState({
    readerReadMode: 'single',
    readerDoublePageMode: false,
    readerAutoReadEnabled: false,
    readerAutoReadPageIntervalMs: 1000
  })
  client.setQueryData(queryKeys.comicDetail('100'), {
    comic: {
      title: 'Test book',
      authors: [],
      image: '',
      chapters: [
        { id: '301', title: 'Chapter 3', sort: '3' },
        { id: '101', title: 'Chapter 1', sort: '1' },
        { id: '201', title: 'Chapter 2', sort: '2' }
      ]
    }
  })
  for (const [id, count] of [
    ['101', 3],
    ['201', 4],
    ['301', 5]
  ] as const) {
    client.setQueryData(queryKeys.readerManifest(id), {
      readId: id,
      pageCount: count,
      pages: Array.from({ length: count }, (_, index) => ({
        index,
        name: index + '.jpg',
        path: '/' + id + '/' + index,
        width: 800,
        height: 1200
      }))
    })
  }
})

afterEach(async () => {
  await act(async () => root.unmount())
  container.remove()
  client.clear()
  notifyManager.setScheduler(defaultScheduler)
  vi.useRealTimers()
  vi.unstubAllGlobals()
  vi.clearAllMocks()
})

async function render(page = 3) {
  await act(async () =>
    root.render(
      <QueryClientProvider client={client}>
        <ReaderPage comicId="101" search={{ albumId: '100', page }} />
      </QueryClientProvider>
    )
  )
}

test('uses reading order despite descending chapter metadata and writes one coherent history snapshot', async () => {
  const legacySettings = { readerDoublePageMode: false, readerChapterTransitions: false }
  useSettingsStore.setState(legacySettings)
  await render()
  await act(async () =>
    container.querySelector<HTMLButtonElement>('[aria-label="下一页"]')!.click()
  )
  expect(container.querySelector('output')!.textContent).toBe('101:2:3')
  const transition = container.querySelector('[data-reader-transition]')!
  expect(transition.textContent).toContain('已读完')
  expect(transition.textContent).toContain('Chapter 1')
  expect(transition.textContent).toContain('下一章')
  expect(transition.textContent).toContain('Chapter 2')
  expect(navigate).not.toHaveBeenCalled()
  expect(useReaderHistorySync).toHaveBeenLastCalledWith(
    expect.objectContaining({ comicId: '101', currentIndex: 2, pageCount: 3 })
  )
  expect(transition.querySelector('button')).toBeNull()
  await act(async () =>
    container.querySelector<HTMLButtonElement>('[aria-label="下一页"]')!.click()
  )
  expect(container.querySelector('output')!.textContent).toBe('201:0:4')
  expect(useReaderHistorySync).toHaveBeenLastCalledWith(
    expect.objectContaining({
      albumId: '100',
      comicId: '201',
      chapter: 'Chapter 2',
      currentIndex: 0,
      pageCount: 4
    })
  )
  expect(navigate).toHaveBeenLastCalledWith(
    expect.objectContaining({
      params: { comicId: '201' },
      search: { albumId: '100', page: 1 },
      replace: true,
      resetScroll: false
    })
  )
  await act(async () =>
    container.querySelector<HTMLButtonElement>('[aria-label="上一页"]')!.click()
  )
  expect(container.querySelector('output')!.textContent).toBe('201:0:4')
  expect(container.querySelector('[data-reader-transition]')!.textContent).toContain('上一章')
  expect(container.querySelector('[data-reader-transition] button')).toBeNull()
  await act(async () =>
    container.querySelector<HTMLButtonElement>('[aria-label="上一页"]')!.click()
  )
  expect(container.querySelector('output')!.textContent).toBe('101:2:3')
})

test('automatic reading waits for the image and uses the same cross-chapter navigation', async () => {
  await render()
  await act(async () => useSettingsStore.setState({ readerAutoReadEnabled: true }))
  await act(async () => vi.advanceTimersByTime(3000))
  expect(container.querySelector('output')!.textContent).toBe('101:2:3')
  await act(async () => {
    for (const image of container.querySelectorAll('img')) {
      Object.defineProperties(image, {
        naturalWidth: { value: 800, configurable: true },
        naturalHeight: { value: 1200, configurable: true }
      })
      image.dispatchEvent(new Event('load'))
    }
  })
  await act(async () => vi.advanceTimersByTime(1000))
  expect(container.querySelector('output')!.textContent).toBe('101:2:3')
  expect(container.querySelector('[data-reader-transition]')).not.toBeNull()
  await act(async () => vi.advanceTimersByTime(1000))
  expect(container.querySelector('output')!.textContent).toBe('201:0:4')
})
