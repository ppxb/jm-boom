import { afterEach, beforeEach, describe, expect, vi, test } from 'vitest'
import {
  defaultScheduler,
  focusManager,
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

import { CACHE } from '@/lib/constants'
import { TooltipProvider } from '@/components/ui/tooltip'
import type { FavoriteListResult } from '@/lib/api/favorite'
import { queryKeys } from '@/lib/query-keys'
import { Route } from './favorites'

const FavoritesPage = Route.options.component!
const favoritesKey = queryKeys.favorites(1)
const emptyFavorites = { items: [], total: 0 }

let dom: Window
let root: Root
let queryClient: QueryClient
let requestedUrls: string[]
let globals: Map<string, PropertyDescriptor | undefined>
let router: ReturnType<typeof createTestRouter>
let serverTotal: number
let respond: ((page: number) => Response | Promise<Response>) | undefined

beforeEach(() => {
  dom = new Window({ url: 'http://localhost' })
  const values = {
    window: dom,
    document: dom.document,
    navigator: dom.navigator,
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
  focusManager.setFocused(true)
  notifyManager.setScheduler(queueMicrotask)
  notifyManager.setNotifyFunction(callback => {
    void act(callback)
  })
  requestedUrls = []
  serverTotal = 0
  respond = undefined
  const mockFetch = async (input: Parameters<typeof fetch>[0]) => {
    requestedUrls.push(String(input))
    const page = Number(new URL(String(input), 'http://localhost').searchParams.get('page'))
    return respond ? respond(page) : Response.json(favoritePage(serverTotal, page))
  }
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => root.unmount())
  queryClient.clear()
  focusManager.setFocused(undefined)
  await dom.happyDOM.close()
  notifyManager.setNotifyFunction(callback => callback())
  notifyManager.setScheduler(defaultScheduler)
  vi.restoreAllMocks()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

async function render(visible = true) {
  await act(async () => {
    root.render(
      <QueryClientProvider client={queryClient}>
        <RouterContextProvider router={router}>
          <TooltipProvider>{visible ? <FavoritesPage /> : null}</TooltipProvider>
        </RouterContextProvider>
      </QueryClientProvider>
    )
  })
}

function createTestRouter() {
  const rootRoute = createRootRoute()
  const comicRoute = createRoute({ getParentRoute: () => rootRoute, path: '/comic/$comicId' })
  return createRouter({
    routeTree: rootRoute.addChildren([comicRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] })
  })
}

function favoritePage(total: number, page: number): FavoriteListResult {
  const start = (page - 1) * 20
  return {
    total,
    items: Array.from({ length: Math.max(0, Math.min(20, total - start)) }, (_, index) => ({
      id: String(start + index + 1),
      title: `Favorite ${start + index + 1}`,
      author: 'Test author',
      description: '',
      image: '',
      tags: [],
      favoritedAt: 1
    }))
  }
}

function seedPages(total: number) {
  serverTotal = total
  for (let page = 1; page <= Math.ceil(total / 20); page += 1) {
    queryClient.setQueryData(queryKeys.favorites(page), favoritePage(total, page))
  }
}

async function nextPage() {
  const next = document.querySelector<HTMLAnchorElement>('[aria-label="Go to next page"]')
  expect(next).not.toBeNull()
  await act(async () => {
    next!.click()
  })
}

async function refreshPage(page: number) {
  await act(async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.favorites(page), exact: true })
  })
}

async function focus() {
  await act(async () => {
    focusManager.setFocused(false)
    focusManager.setFocused(true)
  })
}

describe('favorites query refresh policy', () => {
  test('refreshes a cached list older than ten seconds when entering the page', async () => {
    const updatedAt = Date.now() - 20_000
    queryClient.setQueryData(favoritesKey, emptyFavorites, { updatedAt })
    await render()
    expect(requestedUrls).toEqual(['/api/favorites?page=1'])
    expect(queryClient.getQueryState(favoritesKey)?.dataUpdatedAt).toBeGreaterThan(updatedAt)
  })

  test('refreshes an inactive list invalidated by a local favorite mutation on return', async () => {
    queryClient.setQueryData(favoritesKey, emptyFavorites)
    await render()
    expect(requestedUrls).toHaveLength(0)
    await render(false)
    await queryClient.invalidateQueries({ queryKey: queryKeys.favorites() })
    expect(requestedUrls).toHaveLength(0)
    await render()
    expect(requestedUrls).toEqual(['/api/favorites?page=1'])
    expect(queryClient.getQueryState(favoritesKey)?.isInvalidated).toBe(false)
  })

  test('refreshes a stale mounted list when the window regains focus', async () => {
    queryClient.setQueryData(favoritesKey, emptyFavorites)
    await render()
    await act(async () => {
      queryClient.setQueryData(favoritesKey, emptyFavorites, { updatedAt: Date.now() - 20_000 })
    })
    expect(requestedUrls).toHaveLength(0)
    await focus()
    expect(requestedUrls).toEqual(['/api/favorites?page=1'])
  })

  test('reuses fresh cached data on mount and focus', async () => {
    queryClient.setQueryData(favoritesKey, emptyFavorites)
    await render()
    await focus()
    await render(false)
    await render()
    await focus()
    expect(requestedUrls).toHaveLength(0)
  })
})

describe('favorites pagination', () => {
  test('returns to page one and refreshes its cached total when only twenty favorites remain', async () => {
    seedPages(21)
    await render()
    await nextPage()
    expect(document.body.textContent).toContain('Favorite 21')
    serverTotal = 20
    await refreshPage(2)
    expect(requestedUrls).toEqual(['/api/favorites?page=2', '/api/favorites?page=1'])
    expect(queryClient.getQueryData<FavoriteListResult>(favoritesKey)?.total).toBe(20)
    expect(document.body.textContent).toContain('Favorite 1')
    expect(document.body.textContent).not.toContain('暂无收藏')
    expect(document.querySelector('[data-slot="pagination"]')).toBeNull()
  })

  test('returns to the last valid page when several pages are removed', async () => {
    seedPages(81)
    await render()
    for (let page = 1; page < 5; page += 1) await nextPage()
    serverTotal = 41
    await refreshPage(5)
    expect(requestedUrls).toEqual(['/api/favorites?page=5', '/api/favorites?page=3'])
    expect(document.body.textContent).toContain('第 3 页')
    expect(document.body.textContent).toContain('Favorite 41')
  })

  test('resets an emptied collection to page one without requesting page zero', async () => {
    seedPages(21)
    await render()
    await nextPage()
    serverTotal = 0
    await refreshPage(2)
    expect(requestedUrls).toEqual(['/api/favorites?page=2', '/api/favorites?page=1'])
    expect(document.body.textContent).toContain('暂无收藏')
    expect(document.querySelector('[data-slot="pagination"]')).toBeNull()
  })

  test('keeps the selected page while its first response is pending', async () => {
    serverTotal = 41
    queryClient.setQueryData(favoritesKey, favoritePage(41, 1))
    let finish!: (response: Response) => void
    respond = () =>
      new Promise<Response>(resolve => {
        finish = resolve
      })
    await render()
    await nextPage()
    expect(requestedUrls).toEqual(['/api/favorites?page=2'])
    expect(
      queryClient
        .getQueryCache()
        .find({ queryKey: queryKeys.favorites(2), exact: true })
        ?.isActive()
    ).toBe(true)
    await act(async () => {
      finish(Response.json(favoritePage(41, 2)))
    })
    expect(document.body.textContent).toContain('第 2 页')
    expect(document.body.textContent).toContain('Favorite 21')
  })

  test('retries the same page after a failed request instead of resetting to page one', async () => {
    serverTotal = 41
    queryClient.setQueryData(favoritesKey, favoritePage(41, 1))
    respond = () => Response.json({ error: 'Unavailable', retryable: false }, { status: 503 })
    await render()
    await nextPage()
    expect(document.body.textContent).toContain('收藏加载失败')
    const retry = Array.from(document.querySelectorAll('button')).find(
      button => button.textContent === '重试'
    )
    expect(retry).toBeDefined()
    respond = undefined
    await act(async () => {
      retry!.click()
    })
    expect(requestedUrls).toEqual(['/api/favorites?page=2', '/api/favorites?page=2'])
    expect(document.body.textContent).toContain('第 2 页')
  })
})
