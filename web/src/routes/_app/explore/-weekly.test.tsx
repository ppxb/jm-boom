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
  RouterProvider,
  type AnyRoute
} from '@tanstack/react-router'
import { Window } from 'happy-dom'
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import { TooltipProvider } from '@/components/ui/tooltip'
import type { WeekFiltersResult, WeekItemsResult } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { Route } from './weekly'

const validFilters: WeekFiltersResult = {
  categories: [{ id: '262', time: '2026-10-22', title: 'Test week', label: 'Test week' }],
  types: [{ id: 'manga', title: 'Test type' }],
  defaultCategoryId: '262',
  defaultTypeId: 'manga'
}
const emptyFilters: WeekFiltersResult = {
  categories: [],
  types: [],
  defaultCategoryId: null,
  defaultTypeId: null
}
const weeklyItems: WeekItemsResult = {
  page: 1,
  total: 1,
  items: [{ id: '100', title: 'Weekly comic', author: '', description: '', image: '', tags: [] }]
}

let dom: Window
let root: Root
let queryClient: QueryClient
let globals: Map<string, PropertyDescriptor | undefined>
let requests: string[]
let respondFilters: () => Response | Promise<Response>
let respondItems: () => Response | Promise<Response>
let pendingResponses: Array<(response: Response) => void>

beforeEach(() => {
  dom = new Window({ url: 'http://localhost/explore/weekly' })
  const values = {
    window: dom,
    document: dom.document,
    navigator: dom.navigator,
    HTMLElement: dom.HTMLElement,
    scrollTo: dom.scrollTo.bind(dom),
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
  notifyManager.setScheduler(queueMicrotask)
  requests = []
  pendingResponses = []
  respondFilters = () => Response.json(validFilters)
  respondItems = () => Response.json(weeklyItems)
  const mockFetch = async (input: Parameters<typeof fetch>[0]) => {
    const path = String(input)
    requests.push(path)
    if (path === '/api/home/weekly/filters') return respondFilters()
    if (path.startsWith('/api/home/weekly/items?')) return respondItems()
    throw new Error(`Unexpected request: ${path}`)
  }
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => {
    root.unmount()
    for (const resolve of pendingResponses) resolve(unavailable())
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

async function render(href = '/explore/weekly') {
  const rootRoute = createRootRoute()
  const appRoute = createRoute({ getParentRoute: () => rootRoute, id: '_app' })
  const exploreRoute = createRoute({ getParentRoute: () => appRoute, path: 'explore' })
  const weeklyRoute: AnyRoute = Route
  Object.assign(weeklyRoute.options, {
    getParentRoute: () => exploreRoute,
    path: 'weekly',
    id: '/weekly'
  })
  const comicRoute = createRoute({ getParentRoute: () => appRoute, path: 'comic/$comicId' })
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      appRoute.addChildren([exploreRoute.addChildren([weeklyRoute]), comicRoute])
    ]),
    history: createMemoryHistory({ initialEntries: [href] })
  })
  await act(async () => {
    await router.load()
    root.render(
      <StrictMode>
        <QueryClientProvider client={queryClient}>
          <TooltipProvider>
            <RouterProvider router={router} />
          </TooltipProvider>
        </QueryClientProvider>
      </StrictMode>
    )
  })
}

function deferResponse() {
  let finish!: (response: Response) => void
  const response = new Promise<Response>(resolve => {
    finish = resolve
    pendingResponses.push(resolve)
  })
  return { response, finish }
}

function unavailable() {
  return Response.json({ error: 'Unavailable', retryable: false }, { status: 503 })
}

function itemRequests() {
  return requests.filter(path => path.startsWith('/api/home/weekly/items?'))
}

function expectEmptyFilters() {
  expect(document.body.textContent).toContain('暂无每周推荐')
  expect(document.querySelector('.animate-pulse')).toBeNull()
  expect(document.querySelector('[role="combobox"]')).toBeNull()
}

async function retry() {
  const button = Array.from(document.querySelectorAll('button')).find(
    button => button.textContent?.trim() === '重试'
  )
  expect(button).toBeDefined()
  await act(async () => button!.click())
}

describe('weekly filter loading states', () => {
  test('waits for valid filters before requesting items even when the URL has filter IDs', async () => {
    const filters = deferResponse()
    const items = deferResponse()
    respondFilters = () => filters.response
    respondItems = () => items.response
    await render('/explore/weekly?categoryId=%22262%22&typeId=manga')
    expect(document.querySelector('[data-slot="skeleton"]')).not.toBeNull()
    expect(document.body.textContent).not.toContain('暂无每周推荐')
    expect(itemRequests()).toHaveLength(0)
    await act(async () => filters.finish(Response.json(validFilters)))
    expect(document.querySelectorAll('[role="combobox"]')).toHaveLength(2)
    expect(document.querySelector('[data-slot="skeleton"]')).not.toBeNull()
    expect(itemRequests()).toEqual(['/api/home/weekly/items?page=1&categoryId=262&typeId=manga'])
    await act(async () => items.finish(Response.json(weeklyItems)))
    expect(document.body.textContent).toContain('Weekly comic')
  })

  test.each([
    { label: 'both filter lists', filters: emptyFilters },
    {
      label: 'categories',
      filters: { ...emptyFilters, types: validFilters.types, defaultTypeId: 'manga' }
    },
    {
      label: 'types',
      filters: { ...emptyFilters, categories: validFilters.categories, defaultCategoryId: '262' }
    }
  ])(
    'shows an empty state when $label are empty without using stale URL IDs',
    async ({ filters }) => {
      respondFilters = () => Response.json(filters)
      await render('/explore/weekly?categoryId=%22262%22&typeId=manga')
      expectEmptyFilters()
      expect(itemRequests()).toHaveLength(0)
    }
  )

  test('keeps filter request errors retryable and loads items after a successful retry', async () => {
    respondFilters = unavailable
    await render()
    expect(document.body.textContent).toContain('数据加载失败')
    expect(document.querySelector('.animate-pulse')).toBeNull()
    expect(itemRequests()).toHaveLength(0)
    respondFilters = () => Response.json(validFilters)
    await retry()
    expect(requests.filter(path => path.endsWith('/filters'))).toHaveLength(2)
    expect(itemRequests()).toHaveLength(1)
    expect(document.body.textContent).toContain('Weekly comic')
  })

  test('keeps valid selectors when the item request fails and retries only the items', async () => {
    respondItems = unavailable
    await render()
    expect(document.body.textContent).toContain('数据加载失败')
    expect(document.querySelectorAll('[role="combobox"]')).toHaveLength(2)
    expect(document.querySelector('.animate-pulse')).toBeNull()
    respondItems = () => Response.json(weeklyItems)
    await retry()
    expect(itemRequests()).toHaveLength(2)
    expect(requests.filter(path => path.endsWith('/filters'))).toHaveLength(1)
    expect(document.body.textContent).toContain('Weekly comic')
  })

  test('keeps valid selectors when the item list is empty', async () => {
    respondItems = () => Response.json({ page: 1, total: 0, items: [] })
    await render()
    expect(document.body.textContent).toContain('暂无每周推荐')
    expect(document.querySelectorAll('[role="combobox"]')).toHaveLength(2)
    expect(document.querySelector('.animate-pulse')).toBeNull()
    expect(itemRequests()).toHaveLength(1)
  })

  test('replaces cached items with an empty state when refreshed filters become empty', async () => {
    await render()
    expect(document.body.textContent).toContain('Weekly comic')
    respondFilters = () => Response.json(emptyFilters)
    await act(async () => {
      await queryClient.invalidateQueries({ queryKey: queryKeys.weekFilters() })
    })
    expectEmptyFilters()
    expect(document.body.textContent).not.toContain('Weekly comic')
    expect(itemRequests()).toHaveLength(1)
  })
})
