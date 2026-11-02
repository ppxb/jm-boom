import { afterEach, beforeEach, describe, expect, vi, test } from 'vitest'
import {
  defaultScheduler,
  notifyManager,
  QueryClient,
  QueryClientProvider
} from '@tanstack/react-query'
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterContextProvider
} from '@tanstack/react-router'
import { Window } from 'happy-dom'
import { act } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import { TooltipProvider } from '@/components/ui/tooltip'
import type { ComicDetail } from '@/domain/comic'
import type { ComicStateResult } from '@/lib/api/comic'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { ComicDetailPage } from './page'
import { useComicFavorite } from './use-comic-favorite'

const comic: ComicDetail = {
  id: '100',
  title: 'Test comic',
  description: 'Comic description',
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
  chapters: [{ id: '101', title: 'First chapter', sort: '1' }]
}
const savedState: ComicStateResult = {
  isFavorite: true,
  history: {
    id: '100',
    title: comic.title,
    author: '',
    image: '',
    chapterId: '101',
    chapterTitle: 'First chapter',
    pageIndex: 19,
    pageCount: 40,
    lastReadAt: 123
  }
}
const unknownLabels = new Set(['收藏状态未知', '收藏', '已收藏', '取消收藏'])

let dom: Window
let root: Root
let queryClient: QueryClient
let router: ReturnType<typeof createTestRouter>
let requests: Array<{ path: string; method: string }>
let respondState: () => Response | Promise<Response>
let pendingResponses: Array<(response: Response) => void>
let globals: Map<string, PropertyDescriptor | undefined>

beforeEach(() => {
  dom = new Window({ url: 'http://localhost' })
  const values = {
    window: dom,
    document: dom.document,
    navigator: dom.navigator,
    HTMLElement: dom.HTMLElement,
    requestAnimationFrame: dom.requestAnimationFrame.bind(dom),
    cancelAnimationFrame: dom.cancelAnimationFrame.bind(dom),
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
  router = createTestRouter()
  queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: CACHE.LIST_STALE_TIME,
        gcTime: Infinity,
        refetchOnMount: false,
        refetchOnWindowFocus: false,
        retry: false
      }
    }
  })
  queryClient.setQueryData(queryKeys.comicDetail('100'), { comic })
  notifyManager.setScheduler(queueMicrotask)
  notifyManager.setNotifyFunction(callback => {
    void act(callback)
  })
  requests = []
  pendingResponses = []
  respondState = () => Response.json({ isFavorite: false, history: null })
  const mockFetch = async (
    input: Parameters<typeof fetch>[0],
    init?: Parameters<typeof fetch>[1]
  ) => {
    const path = String(input)
    requests.push({ path, method: init?.method ?? 'GET' })
    if (path === '/api/comics/100/state') return respondState()
    if (path === '/api/reader/101/manifest') return Response.json({ chapter_id: '101', pages: [] })
    if (path === '/api/favorites/100') {
      return init?.method === 'DELETE'
        ? new Response(null, { status: 204 })
        : Response.json({ ...comic, author: '', favoritedAt: 1 })
    }
    throw new Error(`Unexpected request: ${path}`)
  }
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => {
    root.unmount()
    for (const resolve of pendingResponses) resolve(unavailableState())
  })
  queryClient.clear()
  await dom.happyDOM.close()
  notifyManager.setScheduler(defaultScheduler)
  notifyManager.setNotifyFunction(callback => callback())
  vi.restoreAllMocks()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

function createTestRouter() {
  const rootRoute = createRootRoute()
  return createRouter({
    routeTree: rootRoute.addChildren([
      createRoute({ getParentRoute: () => rootRoute, path: '/comic/$comicId' }),
      createRoute({ getParentRoute: () => rootRoute, path: '/reader/$comicId' })
    ]),
    history: createMemoryHistory({ initialEntries: ['/comic/100'] })
  })
}

async function render(content = <ComicDetailPage comicId="100" />) {
  await act(async () => {
    root.render(
      <QueryClientProvider client={queryClient}>
        <RouterContextProvider router={router}>
          <TooltipProvider>{content}</TooltipProvider>
        </RouterContextProvider>
      </QueryClientProvider>
    )
  })
}

function unavailableState() {
  return Response.json({ error: 'Unavailable', retryable: false }, { status: 503 })
}

function deferState() {
  let finish!: (response: Response) => void
  const response = new Promise<Response>(resolve => {
    finish = resolve
    pendingResponses.push(resolve)
  })
  respondState = () => response
  return finish
}

function buttonLabel(button: HTMLButtonElement) {
  return button.getAttribute('aria-label') ?? button.textContent?.trim() ?? ''
}

function favoriteButtons() {
  return Array.from(document.querySelectorAll('button')).filter(button =>
    unknownLabels.has(buttonLabel(button))
  )
}

function readingLinks() {
  return Array.from(document.querySelectorAll('a')).filter(link =>
    /^(开始阅读|继续阅读)$/.test(link.textContent?.trim() ?? '')
  )
}

function favoriteWrites() {
  return requests.filter(request => request.path.startsWith('/api/favorites/'))
}

function retryButton() {
  const button = Array.from(document.querySelectorAll('button')).find(
    button => buttonLabel(button) === '重试状态'
  )
  expect(button).toBeDefined()
  return button!
}

function FavoriteActionHarness() {
  const favorite = useComicFavorite({ comic, state: undefined })
  return <button onClick={favorite.toggle}>Invoke favorite</button>
}

describe('comic detail state', () => {
  test('keeps unknown actions disabled while the first state request is pending', async () => {
    deferState()
    await render()
    expect(document.body.textContent).toContain(comic.title)
    expect(document.querySelector('[role="status"]')?.textContent).toContain(
      '正在读取收藏与阅读进度'
    )
    const buttons = favoriteButtons()
    expect(buttons).toHaveLength(2)
    for (const button of buttons) {
      expect(buttonLabel(button)).toBe('收藏状态未知')
      expect(button.disabled).toBe(true)
    }
    expect(readingLinks()).toHaveLength(0)
    expect(document.body.textContent).toContain('First chapter')
    expect(requests.some(request => request.path.includes('/manifest'))).toBe(false)
  })

  test('shows a state error with manual retry without enabling unknown actions', async () => {
    respondState = unavailableState
    await render()
    expect(document.querySelector('[role="alert"]')?.textContent).toContain(
      '收藏与阅读进度加载失败'
    )
    const buttons = favoriteButtons()
    expect(buttons).toHaveLength(2)
    await act(async () => {
      for (const button of buttons) {
        expect(button.disabled).toBe(true)
        button.click()
      }
    })
    expect(favoriteWrites()).toHaveLength(0)
    expect(readingLinks()).toHaveLength(0)
    expect(requests.some(request => request.path.includes('/manifest'))).toBe(false)
    expect(document.body.textContent).toContain('First chapter')
    expect(retryButton().disabled).toBe(false)
  })

  test('restores favorite state and the saved reading position after retry succeeds', async () => {
    respondState = unavailableState
    await render()
    const finish = deferState()
    await act(async () => retryButton().click())
    expect(requests.filter(request => request.path.endsWith('/state'))).toHaveLength(2)
    expect(favoriteButtons().every(button => button.disabled)).toBe(true)
    await act(async () => finish(Response.json(savedState)))
    expect(document.querySelector('[role="alert"]')).toBeNull()
    expect(favoriteButtons().every(button => !button.disabled)).toBe(true)
    const links = readingLinks()
    expect(links).toHaveLength(2)
    for (const link of links) {
      expect(link.textContent).toBe('继续阅读')
      expect(new URL(link.href).pathname).toBe('/reader/101')
      expect(new URL(link.href).searchParams.get('page')).toBe('20')
    }
    await act(async () => favoriteButtons()[0].click())
    expect(favoriteWrites()).toEqual([{ path: '/api/favorites/100', method: 'DELETE' }])
    expect(
      queryClient.getQueryData<ComicStateResult>(queryKeys.comicState('100'))?.history
    ).toEqual(savedState.history)
  })

  test('keeps cached favorite and reading state when a background refresh fails', async () => {
    queryClient.setQueryData(queryKeys.comicState('100'), savedState, {
      updatedAt: Date.now() - 20_000
    })
    respondState = unavailableState
    await render()
    expect(document.querySelector('[role="alert"]')?.textContent).toContain('上次记录')
    expect(favoriteButtons().map(buttonLabel)).toEqual(['取消收藏', '已收藏'])
    expect(favoriteButtons().every(button => !button.disabled)).toBe(true)
    expect(readingLinks()).toHaveLength(2)
    for (const link of readingLinks())
      expect(new URL(link.href).searchParams.get('page')).toBe('20')
    const finish = deferState()
    await act(async () => retryButton().click())
    expect(readingLinks().every(link => link.textContent === '继续阅读')).toBe(true)
    await act(async () => finish(unavailableState()))
    expect(queryClient.getQueryData<ComicStateResult>(queryKeys.comicState('100'))).toEqual(
      savedState
    )
  })

  test('shows the upstream total page count when available', async () => {
    queryClient.setQueryData(queryKeys.comicDetail('100'), {
      comic: { ...comic, totalPhotos: 128 }
    })
    await render()
    expect(document.body.textContent).toContain('总页数：128')
  })

  test('allows adding a favorite after a confirmed unfavorited response', async () => {
    await render()
    expect(document.querySelector('[role="alert"]')).toBeNull()
    expect(favoriteButtons().every(button => !button.disabled)).toBe(true)
    expect(readingLinks().every(link => link.textContent === '开始阅读')).toBe(true)
    await act(async () => favoriteButtons()[0].click())
    expect(favoriteWrites()).toEqual([{ path: '/api/favorites/100', method: 'PUT' }])
    expect(
      queryClient.getQueryData<ComicStateResult>(queryKeys.comicState('100'))?.isFavorite
    ).toBe(true)
  })

  test('does not submit a favorite mutation when the hook has no state', async () => {
    await render(<FavoriteActionHarness />)
    await act(async () => document.querySelector<HTMLButtonElement>('button')!.click())
    expect(favoriteWrites()).toHaveLength(0)
  })
})
