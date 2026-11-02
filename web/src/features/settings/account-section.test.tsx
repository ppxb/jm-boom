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
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import { TooltipProvider } from '@/components/ui/tooltip'
import type { AccountState } from '@/lib/api/setting'
import { queryKeys } from '@/lib/query-keys'
import { SettingsPage } from './page'

const accountKey = queryKeys.settingsAccount()
const savedAccount = {
  username: 'saved-user',
  autoLogin: true,
  autoSignIn: true,
  loginStatus: 'loggedIn',
  signInStatus: 'pending'
} satisfies AccountState
const emptyAccount: AccountState = {
  ...savedAccount,
  username: null,
  loginStatus: 'loggedOut'
}

let dom: Window
let root: Root
let queryClient: QueryClient
let globals: Map<string, PropertyDescriptor | undefined>
let serverAccount: AccountState
let requests: Array<{ method: string; body: unknown }>
let respondAccount: (method: string, body: unknown) => Response | Promise<Response>
let pendingResponses: Array<(response: Response) => void>

beforeEach(() => {
  dom = new Window({ url: 'http://localhost/settings' })
  const values = {
    window: dom,
    document: dom.document,
    navigator: dom.navigator,
    HTMLElement: dom.HTMLElement,
    Event: dom.Event,
    KeyboardEvent: dom.KeyboardEvent,
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
        gcTime: Infinity,
        retry: false,
        refetchOnMount: false,
        refetchOnWindowFocus: false
      }
    }
  })
  queryClient.setQueryData(accountKey, savedAccount)
  serverAccount = { ...savedAccount }
  requests = []
  pendingResponses = []
  respondAccount = () => Response.json(serverAccount)
  focusManager.setFocused(true)
  notifyManager.setScheduler(queueMicrotask)
  const mockFetch = async (
    input: Parameters<typeof fetch>[0],
    init?: Parameters<typeof fetch>[1]
  ) => {
    const path = String(input)
    if (path === '/api/settings/account') {
      const method = init?.method ?? 'GET'
      const body = typeof init?.body === 'string' ? JSON.parse(init.body) : undefined
      requests.push({ method, body })
      return respondAccount(method, body)
    }
    if (path === '/api/settings/endpoints')
      return Response.json({
        mode: 'auto',
        currentEndpoint: 'https://example.invalid',
        selectedEndpoint: null,
        endpoints: []
      })
    if (path === '/api/settings/system')
      return Response.json({
        serverVersion: 'test',
        cache: { sizeBytes: 0, entryCount: 0, maxSizeBytes: 1024 }
      })
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
  focusManager.setFocused(undefined)
  await dom.happyDOM.close()
  notifyManager.setScheduler(defaultScheduler)
  notifyManager.setNotifyFunction(callback => callback())
  vi.restoreAllMocks()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

async function render() {
  const rootRoute = createRootRoute()
  const router = createRouter({
    routeTree: rootRoute.addChildren([
      createRoute({ getParentRoute: () => rootRoute, path: '/settings' })
    ]),
    history: createMemoryHistory({ initialEntries: ['/settings'] })
  })
  await act(async () => {
    root.render(
      <StrictMode>
        <QueryClientProvider client={queryClient}>
          <RouterContextProvider router={router}>
            <TooltipProvider>
              <SettingsPage />
            </TooltipProvider>
          </RouterContextProvider>
        </QueryClientProvider>
      </StrictMode>
    )
  })
}

function usernameInput() {
  return document.querySelector<HTMLInputElement>('input[autocomplete="username"]')!
}

function passwordInput() {
  return document.querySelector<HTMLInputElement>('input[type="password"]')!
}

function accountSwitches() {
  return Array.from(
    usernameInput().closest('section')!.querySelectorAll<HTMLButtonElement>('[role="switch"]')
  )
}

function formValues() {
  const [autoLogin, autoSignIn] = accountSwitches()
  return {
    username: usernameInput().value,
    password: passwordInput().value,
    autoLogin: autoLogin.getAttribute('aria-checked') === 'true',
    autoSignIn: autoSignIn.getAttribute('aria-checked') === 'true'
  }
}

async function typeInto(input: HTMLInputElement, value: string) {
  await act(async () => {
    // React DOM loads before the test document and uses its keyboard input fallback.
    Object.assign(input, { attachEvent() {}, detachEvent() {} })
    input.focus()
    Object.getOwnPropertyDescriptor(dom.HTMLInputElement.prototype, 'value')!.set!.call(
      input,
      value
    )
    input.dispatchEvent(new Event('input', { bubbles: true }))
    input.dispatchEvent(new KeyboardEvent('keyup', { bubbles: true, key: 'a' }))
  })
}

async function editDraft() {
  await typeInto(usernameInput(), 'draft-user')
  await typeInto(passwordInput(), 'draft-password')
  await act(async () => {
    for (const button of accountSwitches()) button.click()
  })
  const draft = {
    username: 'draft-user',
    password: 'draft-password',
    autoLogin: false,
    autoSignIn: false
  }
  expect(formValues()).toEqual(draft)
  return draft
}

async function clickButton(label: string) {
  const button = Array.from(document.querySelectorAll('button')).find(
    button => button.textContent?.trim() === label
  )!
  expect(button).toBeDefined()
  expect(button.disabled).toBe(false)
  await act(async () => button.click())
}

async function refreshAccount() {
  await act(async () => {
    await queryClient.refetchQueries({ queryKey: accountKey, exact: true })
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

describe('account form drafts', () => {
  test('initializes the form when the first account response arrives', async () => {
    queryClient.removeQueries({ queryKey: accountKey })
    const pending = deferResponse()
    respondAccount = () => pending.response
    await render()
    expect(usernameInput().disabled).toBe(true)
    expect(usernameInput().value).toBe('')
    await act(async () => {
      pending.finish(Response.json({ ...savedAccount, autoLogin: false, autoSignIn: false }))
    })
    expect(usernameInput().disabled).toBe(false)
    expect(formValues()).toEqual({
      username: savedAccount.username,
      password: '',
      autoLogin: false,
      autoSignIn: false
    })
  })

  test('preserves every draft field while login and sign-in states refresh', async () => {
    await render()
    const draft = await editDraft()
    serverAccount = { ...savedAccount, loginStatus: 'loggingIn', signInStatus: 'signingIn' }
    await refreshAccount()
    expect(formValues()).toEqual(draft)
    expect(document.body.textContent).toContain('正在登录')
    expect(document.body.textContent).toContain('正在签到')
    serverAccount = { ...savedAccount, signInStatus: 'signedIn' }
    await refreshAccount()
    expect(formValues()).toEqual(draft)
    expect(document.body.textContent).toContain('已登录')
    expect(document.body.textContent).toContain('已签到')
  })

  test('preserves drafts when an expired account query refreshes on window focus', async () => {
    await render()
    const draft = await editDraft()
    await act(async () => {
      focusManager.setFocused(false)
      await queryClient.invalidateQueries({ queryKey: accountKey, refetchType: 'none' })
    })
    const requestCount = requests.length
    serverAccount = { ...savedAccount, signInStatus: 'signedIn' }
    await act(async () => focusManager.setFocused(true))
    expect(requests.length).toBe(requestCount + 1)
    expect(formValues()).toEqual(draft)
    expect(document.body.textContent).toContain('已签到')
  })

  test('clears the password after saving even when the response matches the cached state', async () => {
    await render()
    await typeInto(passwordInput(), 'new-password')
    const cachedAccount = queryClient.getQueryData<AccountState>(accountKey)
    const pending = deferResponse()
    respondAccount = method => (method === 'PUT' ? pending.response : Response.json(serverAccount))
    await clickButton('保存并登录')
    expect(requests[requests.length - 1]).toEqual({
      method: 'PUT',
      body: {
        username: savedAccount.username,
        password: 'new-password',
        autoLogin: true,
        autoSignIn: true
      }
    })
    expect(passwordInput().disabled).toBe(true)
    expect(passwordInput().value).toBe('new-password')
    await act(async () => pending.finish(Response.json(savedAccount)))
    expect(queryClient.getQueryData<AccountState>(accountKey)).toBe(cachedAccount)
    expect(passwordInput().disabled).toBe(false)
    expect(passwordInput().value).toBe('')
  })

  test('retains a failed save through background updates and resets after a successful retry', async () => {
    await render()
    const draft = await editDraft()
    const pending = deferResponse()
    respondAccount = method => (method === 'PUT' ? pending.response : Response.json(serverAccount))
    await clickButton('保存并登录')
    serverAccount = { ...savedAccount, loginStatus: 'loggingIn' }
    await refreshAccount()
    expect(formValues()).toEqual(draft)
    await act(async () => pending.finish(unavailable()))
    expect(formValues()).toEqual(draft)
    expect(passwordInput().disabled).toBe(false)
    serverAccount = { ...savedAccount, username: 'draft-user', autoLogin: false, autoSignIn: false }
    respondAccount = () => Response.json(serverAccount)
    await clickButton('保存并登录')
    expect(formValues()).toEqual({ ...draft, password: '' })
    expect(requests.filter(request => request.method === 'PUT')).toHaveLength(2)
  })

  test('replaces the draft when the saved account identity changes', async () => {
    await render()
    await editDraft()
    serverAccount = { ...savedAccount, username: 'other-user', autoLogin: false }
    await refreshAccount()
    expect(formValues()).toEqual({
      username: 'other-user',
      password: '',
      autoLogin: false,
      autoSignIn: true
    })
  })

  test('keeps the draft after a failed logout and resets it after successful logout', async () => {
    await render()
    const draft = await editDraft()
    respondAccount = () => unavailable()
    await clickButton('退出登录')
    expect(formValues()).toEqual(draft)
    serverAccount = emptyAccount
    respondAccount = () => Response.json(serverAccount)
    await clickButton('退出登录')
    expect(formValues()).toEqual({
      username: '',
      password: '',
      autoLogin: true,
      autoSignIn: true
    })
    expect(requests.filter(request => request.method === 'DELETE')).toHaveLength(2)
  })
})
