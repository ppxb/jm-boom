import { afterEach, beforeEach, describe, expect, vi, test } from 'vitest'
import {
  createMemoryHistory,
  createRootRoute,
  createRouter,
  RouterProvider,
  type AnyRoute
} from '@tanstack/react-router'
import { Window } from 'happy-dom'
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import * as accessGate from '@/lib/access-gate'
import { Route } from './login'

const origin = 'https://reader.example:8443'
const readingTarget = '/reader/101?albumId=100&page=20#page-20'

let dom: Window
let root: Root
let globals: Map<string, PropertyDescriptor | undefined>
let inputMethods: Map<string, PropertyDescriptor | undefined>
let redirects: string[]
let gateEnabled: boolean
let requests: Array<{ path: string; body: unknown }>
let respondLogin: () => Response

beforeEach(() => {
  dom = new Window({ url: `${origin}/login` })
  const values = {
    window: dom,
    document: dom.document,
    navigator: dom.navigator,
    sessionStorage: dom.sessionStorage,
    HTMLElement: dom.HTMLElement,
    Event: dom.Event,
    KeyboardEvent: dom.KeyboardEvent,
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
  // React DOM loads before happy-dom and uses its keyboard change-event fallback.
  inputMethods = new Map(
    ['attachEvent', 'detachEvent'].map(key => [
      key,
      Object.getOwnPropertyDescriptor(dom.HTMLInputElement.prototype, key)
    ])
  )
  for (const key of inputMethods.keys()) {
    Object.defineProperty(dom.HTMLInputElement.prototype, key, {
      value: () => {},
      configurable: true
    })
  }
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
  redirects = []
  requests = []
  gateEnabled = false
  respondLogin = () => Response.json({ success: true })
  vi.spyOn(accessGate, 'loadAccessGateConfig').mockImplementation(async () => ({
    enabled: gateEnabled
  }))
  vi.spyOn(dom.location, 'replace').mockImplementation(target => {
    redirects.push(String(target))
  })
  const mockFetch = async (
    input: Parameters<typeof fetch>[0],
    init?: Parameters<typeof fetch>[1]
  ) => {
    const path = String(input)
    if (path !== '/api/auth/login') throw new Error(`Unexpected request: ${path}`)
    requests.push({ path, body: JSON.parse(String(init?.body)) })
    return respondLogin()
  }
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => root.unmount())
  await dom.happyDOM.close()
  vi.restoreAllMocks()
  for (const [key, descriptor] of inputMethods) {
    if (descriptor) Object.defineProperty(dom.HTMLInputElement.prototype, key, descriptor)
    else Reflect.deleteProperty(dom.HTMLInputElement.prototype, key)
  }
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

async function render(redirect?: string) {
  const rootRoute = createRootRoute()
  const loginRoute: AnyRoute = Route
  Object.assign(loginRoute.options, {
    getParentRoute: () => rootRoute,
    path: 'login',
    id: '/login'
  })
  const search = redirect === undefined ? '' : `?${new URLSearchParams({ redirect })}`
  const router = createRouter({
    routeTree: rootRoute.addChildren([loginRoute]),
    history: createMemoryHistory({ initialEntries: [`/login${search}`] })
  })
  await act(async () => {
    await router.load()
    root.render(
      <StrictMode>
        <RouterProvider router={router} />
      </StrictMode>
    )
  })
}

function destination() {
  expect(redirects).toHaveLength(1)
  const target = new URL(redirects[0], `${origin}/login`)
  expect(target.origin).toBe(origin)
  return target.href
}

async function submitPassword() {
  const input = document.querySelector<HTMLInputElement>('#access-password')!
  expect(input).not.toBeNull()
  await act(async () => {
    input.focus()
    Object.getOwnPropertyDescriptor(dom.HTMLInputElement.prototype, 'value')!.set!.call(
      input,
      'test-password'
    )
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keyup', { bubbles: true, key: 'a' }))
  })
  await act(async () => {
    document
      .querySelector('form')!
      .dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }))
  })
}

describe('access gate redirects', () => {
  test.each([
    { label: 'backslash host', target: '/\\outside.example' },
    { label: 'mixed separators', target: '/\\/outside.example' },
    { label: 'protocol-relative host', target: '//outside.example' },
    { label: 'extra leading slashes', target: '///outside.example' },
    { label: 'tab normalization', target: '/\t/outside.example' },
    { label: 'newline normalization', target: '/\r\n/outside.example' },
    { label: 'userinfo host spoof', target: '/\\reader.example:8443@outside.example' },
    { label: 'hostname prefix spoof', target: '/\\reader.example.outside.example:8443' },
    { label: 'different port', target: '/\\reader.example:8444' },
    { label: 'different protocol', target: 'http://reader.example:8443/bookshelf' }
  ])('rejects $label after the router decodes the redirect parameter', async ({ target }) => {
    await render(target)
    expect(destination()).toBe(`${origin}/bookshelf`)
    expect(requests).toHaveLength(0)
  })

  test.each([
    { target: '/explore', path: '/explore' },
    { target: readingTarget, path: readingTarget },
    {
      target: '/explore/search?keyword=%E6%BC%AB%E7%94%BB#results',
      path: '/explore/search?keyword=%E6%BC%AB%E7%94%BB#results'
    },
    { target: '/folder/..//outside.example/reader', path: '//outside.example/reader' },
    { target: '/%2F%2Foutside.example', path: '/%2F%2Foutside.example' }
  ])(
    'preserves a safe destination $target without reparsing it as an external URL',
    async ({ target, path }) => {
      await render(target)
      expect(destination()).toBe(`${origin}${path}`)
    }
  )

  test.each([
    undefined,
    '',
    'bookshelf',
    'https://outside.example',
    'javascript:alert(1)',
    '/\\[',
    '/bad%escape',
    '/login?redirect=/explore',
    '/folder/../login',
    '/%6Cogin',
    '/LOGIN',
    '/%2Flogin'
  ])('falls back for a missing, invalid, or login-loop destination %s', async target => {
    await render(target)
    expect(destination()).toBe(`${origin}/bookshelf`)
  })

  test('applies the same origin check when access was already granted', async () => {
    gateEnabled = true
    accessGate.grantAccessGate()
    await render('/\\outside.example')
    expect(destination()).toBe(`${origin}/bookshelf`)
    expect(document.querySelector('form')).toBeNull()
    expect(requests).toHaveLength(0)
  })

  test.each([
    { target: '/\\outside.example', path: '/bookshelf' },
    { target: readingTarget, path: readingTarget }
  ])('uses the validated target after password login for $target', async ({ target, path }) => {
    gateEnabled = true
    await render(target)
    expect(redirects).toHaveLength(0)
    await submitPassword()
    expect(requests).toEqual([{ path: '/api/auth/login', body: { password: 'test-password' } }])
    expect(accessGate.hasAccessGateGrant()).toBe(true)
    expect(destination()).toBe(`${origin}${path}`)
  })

  test('does not redirect or grant access after an unsuccessful password login', async () => {
    gateEnabled = true
    respondLogin = () => Response.json({ success: false })
    await render(readingTarget)
    await submitPassword()
    expect(redirects).toHaveLength(0)
    expect(accessGate.hasAccessGateGrant()).toBe(false)
    expect(document.body.textContent).toContain('访问密码错误')
  })
})
