import { afterEach, beforeEach, describe, expect, vi, test } from 'vitest'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Window } from 'happy-dom'
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import { getComicReadManifest, type ComicReadManifestResult } from '@/lib/api/reader'
import { queryKeys } from '@/lib/query-keys'
import * as readerPreload from '@/lib/reader-preload'
import { useNextChapterPrefetch } from './use-next-chapter-prefetch'

type PrefetchProps = Parameters<typeof useNextChapterPrefetch>[0]
type ManifestRequest = {
  path: string
  resolve: (response: Response) => void
  reject: (error: Error) => void
}

const initialProps: PrefetchProps = {
  currentIndex: 79,
  pageCount: 100,
  nextChapter: { id: '201', title: 'Next chapter' }
}

let dom: Window
let root: Root | undefined
let queryClient: QueryClient
let requests: ManifestRequest[]
let preloads: Array<{ scope: string; paths: string[] }>
let clearedScopes: string[]
let globals: Map<string, PropertyDescriptor | undefined>

beforeEach(() => {
  vi.useFakeTimers({ now: new Date('2026-10-22T12:00:00Z') })
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
  Object.defineProperties(dom, {
    setTimeout: { value: globalThis.setTimeout.bind(globalThis), configurable: true },
    clearTimeout: { value: globalThis.clearTimeout.bind(globalThis), configurable: true }
  })
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
  queryClient = new QueryClient({ defaultOptions: { queries: { gcTime: Infinity, retry: false } } })
  requests = []
  preloads = []
  clearedScopes = []
  vi.spyOn(readerPreload, 'setReaderPreloadScope').mockImplementation((scope, paths) => {
    preloads.push({ scope, paths })
  })
  vi.spyOn(readerPreload, 'clearReaderPreloadScope').mockImplementation(scope => {
    clearedScopes.push(scope)
  })
  const mockFetch = (input: Parameters<typeof fetch>[0]) =>
    new Promise<Response>((resolve, reject) => {
      requests.push({ path: String(input), resolve, reject })
    })
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => {
    root?.unmount()
    root = undefined
    for (const request of requests) request.resolve(unavailable())
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

function Reader(props: PrefetchProps) {
  useNextChapterPrefetch(props)
  return null
}

async function render(props = initialProps, strict = false) {
  await act(async () => {
    const reader = (
      <QueryClientProvider client={queryClient}>
        <Reader {...props} />
      </QueryClientProvider>
    )
    root!.render(strict ? <StrictMode>{reader}</StrictMode> : reader)
  })
}

async function unmount() {
  await act(async () => {
    root!.unmount()
    root = undefined
  })
}

async function advance(milliseconds: number) {
  await act(async () => {
    vi.advanceTimersByTime(milliseconds)
  })
}

function manifest(readId = '201'): ComicReadManifestResult {
  return {
    readId,
    pageCount: 10,
    pages: Array.from({ length: 10 }, (_, index) => ({
      index,
      name: `${index + 1}.webp`,
      path: `/api/reader/${readId}/pages/${index}`
    }))
  }
}

async function succeed(request: ManifestRequest, readId = '201') {
  const data = manifest(readId)
  await act(async () => {
    request.resolve(
      Response.json({
        chapter_id: data.readId,
        pages: data.pages.map(page => ({ index: page.index, name: page.name, url: page.path }))
      })
    )
  })
}

function unavailable() {
  return Response.json({ error: 'Unavailable', retryable: true }, { status: 503 })
}

async function fail(request: ManifestRequest) {
  await act(async () => request.resolve(unavailable()))
}

function expectedPreload(readId = '201') {
  return {
    scope: `next-chapter:${readId}`,
    paths: manifest(readId)
      .pages.slice(0, 6)
      .map(page => page.path)
  }
}

describe('next chapter prefetch lifecycle', () => {
  test('starts only when eligible and reuses pending requests and fresh manifests', async () => {
    await render({ ...initialProps, currentIndex: 0 })
    expect(requests).toHaveLength(0)
    await render()
    expect(requests).toHaveLength(1)
    await render({ ...initialProps, currentIndex: 80 })
    expect(requests).toHaveLength(1)
    await succeed(requests[0])
    expect(preloads).toEqual([expectedPreload()])
    expect(
      queryClient.getQueryData<ComicReadManifestResult>(queryKeys.readerManifest('201'))
    ).toEqual(manifest())
    await render({ ...initialProps, currentIndex: 0 })
    await render()
    expect(requests).toHaveLength(1)
    expect(preloads[preloads.length - 1]).toEqual(expectedPreload())
  })

  test('retries a network failure without another render and preloads images only after success', async () => {
    await render()
    await act(async () => requests[0].reject(new TypeError('Network unavailable')))
    expect(queryClient.getQueryState(queryKeys.readerManifest('201'))?.status).toBe('error')
    expect(preloads).toHaveLength(0)
    await advance(4999)
    expect(requests).toHaveLength(1)
    await advance(1)
    expect(requests).toHaveLength(2)
    expect(requests[1].path).toBe('/api/reader/201/manifest')
    await succeed(requests[1])
    expect(preloads).toEqual([expectedPreload()])
    await advance(60_000)
    expect(requests).toHaveLength(2)
  })

  test('limits transient failures to two retries with five and ten second delays', async () => {
    await render()
    await fail(requests[0])
    await advance(5000)
    expect(requests).toHaveLength(2)
    await fail(requests[1])
    await advance(9999)
    expect(requests).toHaveLength(2)
    await advance(1)
    expect(requests).toHaveLength(3)
    await fail(requests[2])
    await render({ ...initialProps, currentIndex: 95 })
    await advance(60_000)
    expect(requests).toHaveLength(3)
    expect(preloads).toHaveLength(0)
  })

  test('does not retry permanent API errors or preload from a failed request', async () => {
    await render()
    await act(async () => {
      requests[0].resolve(
        Response.json({ error: 'Invalid chapter', retryable: false }, { status: 400 })
      )
    })
    await advance(60_000)
    expect(requests).toHaveLength(1)
    expect(preloads).toHaveLength(0)
  })

  test('stops retries below the threshold and allows a new attempt when eligible again', async () => {
    await render()
    await fail(requests[0])
    await render({ ...initialProps, currentIndex: 0 })
    await advance(60_000)
    expect(requests).toHaveLength(1)
    await render()
    expect(requests).toHaveLength(2)
    await succeed(requests[1])
    expect(preloads).toEqual([expectedPreload()])
  })

  test('clears the previous chapter retry timer when the next chapter changes', async () => {
    await render()
    await fail(requests[0])
    await render({ ...initialProps, nextChapter: { id: '202', title: 'Another chapter' } })
    expect(requests).toHaveLength(2)
    expect(requests[1].path).toBe('/api/reader/202/manifest')
    await succeed(requests[1], '202')
    await advance(60_000)
    expect(requests).toHaveLength(2)
    expect(preloads).toEqual([expectedPreload('202')])
    expect(clearedScopes).toContain('next-chapter:201')
  })

  test('ignores an obsolete chapter response without disrupting the new chapter retry', async () => {
    await render()
    await render({ ...initialProps, nextChapter: { id: '202', title: 'Another chapter' } })
    await fail(requests[1])
    await succeed(requests[0])
    expect(preloads).toHaveLength(0)
    await advance(5000)
    expect(requests).toHaveLength(3)
    expect(requests[2].path).toBe('/api/reader/202/manifest')
    await succeed(requests[2], '202')
    expect(preloads).toEqual([expectedPreload('202')])
  })

  test('does not cancel a shared manifest request or preload after unmount', async () => {
    await render()
    const sharedRead = queryClient.fetchQuery({
      queryKey: queryKeys.readerManifest('201'),
      queryFn: () => getComicReadManifest({ readId: '201' })
    })
    expect(requests).toHaveLength(1)
    await unmount()
    await succeed(requests[0])
    expect(await sharedRead).toEqual(manifest())
    expect(preloads).toHaveLength(0)
    expect(clearedScopes).toContain('next-chapter:201')
    await advance(60_000)
    expect(requests).toHaveLength(1)
  })

  test('clears a pending retry when the reader unmounts', async () => {
    await render()
    await fail(requests[0])
    await unmount()
    await advance(60_000)
    expect(requests).toHaveLength(1)
    expect(preloads).toHaveLength(0)
  })

  test('preloads after Strict Mode replays effects without duplicating the request', async () => {
    await render(initialProps, true)
    expect(requests).toHaveLength(1)
    await fail(requests[0])
    expect(preloads).toHaveLength(0)
    await advance(5000)
    expect(requests).toHaveLength(2)
    await succeed(requests[1])
    expect(preloads).toEqual([expectedPreload()])
  })
})
