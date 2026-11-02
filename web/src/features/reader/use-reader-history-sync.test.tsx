import { afterEach, beforeEach, describe, expect, vi, test } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Window } from 'happy-dom'
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import type { ComicStateResult } from '@/lib/api/comic'
import type { ReadingHistoryItem } from '@/lib/api/history'
import { queryKeys } from '@/lib/query-keys'
import { useReaderHistorySync } from './use-reader-history-sync'

type HistoryProps = Parameters<typeof useReaderHistorySync>[0]
type SaveRequest = {
  item: ReadingHistoryItem
  keepalive: boolean
  resolve: (response: Response) => void
  reject: (error: Error) => void
}

const initialProps: HistoryProps = {
  comicId: '101',
  albumId: '100',
  title: 'Test comic',
  author: 'Test author',
  coverUrl: '/api/covers/100',
  chapter: 'Chapter 1',
  currentIndex: 0,
  pageCount: 20
}

let dom: Window
let root: Root | undefined
let queryClient: QueryClient
let requests: SaveRequest[]
let globals: Map<string, PropertyDescriptor | undefined>

beforeEach(() => {
  vi.useFakeTimers({ now: new Date('2026-10-21T12:00:00Z') })
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
  // The hook uses browser timers; drive them with Bun's deterministic clock.
  Object.defineProperties(dom, {
    setTimeout: { value: globalThis.setTimeout.bind(globalThis), configurable: true },
    clearTimeout: { value: globalThis.clearTimeout.bind(globalThis), configurable: true }
  })
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
  queryClient = new QueryClient({ defaultOptions: { queries: { gcTime: Infinity, retry: false } } })
  queryClient.setQueryData<ComicStateResult>(queryKeys.comicState('100'), {
    isFavorite: true,
    history: null
  })
  queryClient.setQueryData(queryKeys.readingHistory(1), { items: [], total: 0 })
  requests = []
  const mockFetch = (_input: Parameters<typeof fetch>[0], init?: Parameters<typeof fetch>[1]) =>
    new Promise<Response>((resolve, reject) => {
      requests.push({
        item: { ...JSON.parse(String(init?.body)), id: '100' },
        keepalive: init?.keepalive ?? false,
        resolve,
        reject
      })
    })
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => {
    root?.unmount()
    root = undefined
    for (const request of requests) request.resolve(new Response(null, { status: 204 }))
  })
  queryClient.clear()
  vi.clearAllTimers()
  vi.useRealTimers()
  await dom.happyDOM.close()
  vi.restoreAllMocks()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

function Reader(props: HistoryProps) {
  useReaderHistorySync(props)
  return null
}

async function render(props = initialProps, strict = false) {
  await act(async () => {
    const reader = <Reader key={props.comicId} {...props} />
    root!.render(
      <QueryClientProvider client={queryClient}>
        {strict ? <StrictMode>{reader}</StrictMode> : reader}
      </QueryClientProvider>
    )
  })
}

async function advance(milliseconds: number) {
  await act(async () => {
    vi.advanceTimersByTime(milliseconds)
  })
}

async function succeed(request: SaveRequest) {
  await act(async () => {
    request.resolve(new Response(null, { status: 204 }))
  })
}

async function fail(request: SaveRequest) {
  await act(async () => {
    request.reject(new TypeError('Network unavailable'))
  })
}

function savedHistory() {
  return queryClient.getQueryData<ComicStateResult>(queryKeys.comicState('100'))?.history
}

describe('useReaderHistorySync', () => {
  test('retries failed progress without another page change and preserves its timestamp', async () => {
    await render()
    await advance(0)
    expect(requests).toHaveLength(1)
    await fail(requests[0])
    await advance(4999)
    expect(requests).toHaveLength(1)
    await advance(1)
    expect(requests).toHaveLength(2)
    expect(requests[1].item).toEqual(requests[0].item)
    await succeed(requests[1])
    await advance(60_000)
    expect(requests).toHaveLength(2)
  })

  test('backs off repeated network failures up to thirty seconds', async () => {
    await render()
    await advance(0)
    for (const delay of [5000, 10_000, 20_000, 30_000, 30_000]) {
      const count = requests.length
      await fail(requests[count - 1])
      await advance(delay - 1)
      expect(requests).toHaveLength(count)
      await advance(1)
      expect(requests).toHaveLength(count + 1)
      expect(requests[count].item.lastReadAt).toBe(requests[0].item.lastReadAt)
    }
  })

  test('does not automatically retry permanent API errors', async () => {
    await render()
    await advance(0)
    await act(async () => {
      requests[0].resolve(
        new Response(JSON.stringify({ error: 'Invalid progress', retryable: false }), {
          status: 400
        })
      )
    })
    await advance(60_000)
    expect(requests).toHaveLength(1)
  })

  test('coalesces page changes while saving and keeps new progress after an older success', async () => {
    await render()
    await advance(0)
    await render({ ...initialProps, currentIndex: 1 })
    await render({ ...initialProps, currentIndex: 2 })
    await advance(10_000)
    expect(requests).toHaveLength(1)
    await succeed(requests[0])
    await advance(0)
    expect(requests).toHaveLength(2)
    expect(requests[1].item.pageIndex).toBe(2)
    expect(requests[1].item.lastReadAt).toBeGreaterThan(requests[0].item.lastReadAt)
    await succeed(requests[1])
    await advance(30_000)
    expect(requests).toHaveLength(2)
  })

  test('keeps the newest progress when an older request fails', async () => {
    await render()
    await advance(0)
    await render({ ...initialProps, currentIndex: 4 })
    await fail(requests[0])
    await advance(5000)
    expect(requests).toHaveLength(2)
    expect(requests[1].item.pageIndex).toBe(4)
  })

  test('uses keepalive on pagehide even while a save is pending and deduplicates that flush', async () => {
    await render()
    await advance(0)
    await act(async () => {
      dom.dispatchEvent(new dom.Event('pagehide'))
      dom.dispatchEvent(new dom.Event('pagehide'))
    })
    expect(requests).toHaveLength(2)
    expect(requests[1].keepalive).toBe(true)
    expect(requests[1].item).toEqual(requests[0].item)
    await succeed(requests[1])
    await fail(requests[0])
    await advance(30_000)
    expect(requests).toHaveLength(2)
  })

  test('flushes failed progress on unmount and stops background retries', async () => {
    await render()
    await advance(0)
    await fail(requests[0])
    await act(async () => {
      root!.unmount()
      root = undefined
    })
    expect(requests).toHaveLength(2)
    expect(requests[1].keepalive).toBe(true)
    expect(requests[1].item).toEqual(requests[0].item)
    await fail(requests[1])
    await advance(60_000)
    expect(requests).toHaveLength(2)
  })

  test('updates cached progress and invalidates history only after the save succeeds', async () => {
    await render()
    await advance(0)
    expect(savedHistory()).toBeNull()
    expect(queryClient.getQueryState(queryKeys.readingHistory(1))?.isInvalidated).toBe(false)
    await succeed(requests[0])
    expect(savedHistory()?.pageIndex).toBe(0)
    expect(queryClient.getQueryState(queryKeys.readingHistory(1))?.isInvalidated).toBe(true)
  })

  test('does not regress a newer chapter after an older session finishes saving', async () => {
    await render()
    await advance(0)
    await render({ ...initialProps, comicId: '102', chapter: 'Chapter 2' })
    await advance(0)
    const latest = requests.find(request => request.item.chapterId === '102')!
    expect(latest).toBeDefined()
    expect(latest.item.lastReadAt).toBeGreaterThan(requests[0].item.lastReadAt)
    await succeed(latest)
    for (const old of requests.filter(request => request !== latest)) await succeed(old)
    expect(savedHistory()?.chapterId).toBe('102')
  })

  test('continues saving after Strict Mode replays the effect lifecycle', async () => {
    await render(initialProps, true)
    for (const request of requests) await succeed(request)
    await advance(5000)
    const active = requests[requests.length - 1]
    await fail(active)
    const count = requests.length
    await advance(5000)
    expect(requests).toHaveLength(count + 1)
    await succeed(requests[requests.length - 1])
  })
})
