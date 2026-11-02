import { afterEach, beforeEach, describe, expect, test } from 'vitest'
import { Window } from 'happy-dom'
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import type { EndpointState } from '@/lib/api/setting'
import { ApiEndpointSection } from './api-endpoint-section'

type SectionProps = Parameters<typeof ApiEndpointSection>[0]

const selectedEndpoint = 'https://manual.example'
const fallbackEndpoint = 'https://fallback.example'
const failedState: EndpointState = {
  mode: 'manual',
  selectedEndpoint,
  currentEndpoint: fallbackEndpoint,
  endpoints: [
    { endpoint: selectedEndpoint, available: false, latencyMs: null, error: 'Request failed' },
    { endpoint: fallbackEndpoint, available: true, latencyMs: 120, error: null }
  ]
}

let dom: Window
let root: Root
let globals: Map<string, PropertyDescriptor | undefined>
let changes: Array<string | null>
let refreshCount: number

beforeEach(() => {
  dom = new Window({ url: 'http://localhost/settings' })
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
  changes = []
  refreshCount = 0
})

afterEach(async () => {
  await act(async () => root.unmount())
  await dom.happyDOM.close()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

async function render(overrides: Partial<SectionProps> = {}) {
  await act(async () => {
    root.render(
      <StrictMode>
        <ApiEndpointSection
          state={failedState}
          isLoading={false}
          isRefreshing={false}
          isChanging={false}
          onEndpointChange={endpoint => changes.push(endpoint)}
          onRefresh={() => refreshCount++}
          {...overrides}
        />
      </StrictMode>
    )
  })
}

function trigger() {
  return document.querySelector<HTMLButtonElement>('[role="combobox"]')!
}

function currentEndpointNotice() {
  return document.querySelector('[role="status"]')
}

describe('endpoint display identity', () => {
  test('shows the selected endpoint failure and names the actual endpoint separately', async () => {
    await render()
    expect(trigger().textContent).toContain('manual.example')
    expect(trigger().textContent).toContain('不可用')
    expect(trigger().textContent).not.toContain('120 ms')
    expect(trigger().textContent).not.toContain('fallback.example')
    expect(currentEndpointNotice()?.textContent).toBe('当前使用：fallback.example')
    expect(changes).toEqual([])
  })

  test('uses the selected endpoint latency when both endpoints have successful probe results', async () => {
    await render({
      state: {
        ...failedState,
        endpoints: [
          { endpoint: selectedEndpoint, available: true, latencyMs: 80, error: null },
          failedState.endpoints[1]
        ]
      }
    })
    expect(trigger().textContent).toContain('manual.example')
    expect(trigger().textContent).toContain('80 ms')
    expect(trigger().textContent).not.toContain('120 ms')
    expect(currentEndpointNotice()?.textContent).toBe('当前使用：fallback.example')
  })

  test('shows untested when the selected endpoint has no probe result', async () => {
    await render({ state: { ...failedState, endpoints: [failedState.endpoints[1]] } })
    expect(trigger().textContent).toContain('manual.example')
    expect(trigger().textContent).toContain('未测试')
    expect(trigger().textContent).not.toContain('120 ms')
    expect(currentEndpointNotice()?.textContent).toBe('当前使用：fallback.example')
  })

  test('removes the extra notice when the selected endpoint becomes active again', async () => {
    await render()
    expect(currentEndpointNotice()).not.toBeNull()
    await render({
      state: {
        ...failedState,
        currentEndpoint: selectedEndpoint,
        endpoints: [
          { endpoint: selectedEndpoint, available: true, latencyMs: 80, error: null },
          failedState.endpoints[1]
        ]
      }
    })
    expect(trigger().textContent).toContain('manual.example')
    expect(trigger().textContent).toContain('80 ms')
    expect(currentEndpointNotice()).toBeNull()
    expect(changes).toEqual([])
  })

  test('keeps the automatic selection display without a duplicate current endpoint notice', async () => {
    await render({ state: { ...failedState, mode: 'auto', selectedEndpoint: null } })
    expect(trigger().textContent).toContain('自动优选')
    expect(trigger().textContent).toContain('fallback.example')
    expect(trigger().textContent).not.toContain('manual.example')
    expect(currentEndpointNotice()).toBeNull()
    expect(changes).toEqual([])
  })

  test('preserves loading states and the refresh action', async () => {
    await render({ state: undefined, isLoading: true })
    expect(trigger().disabled).toBe(true)
    expect(currentEndpointNotice()).toBeNull()
    const refresh = document.querySelector<HTMLButtonElement>('[aria-label="重新测速"]')!
    expect(refresh.disabled).toBe(true)
    await render()
    expect(trigger().disabled).toBe(false)
    await act(async () => refresh.click())
    expect(refreshCount).toBe(1)
    expect(changes).toEqual([])
    await render({ isChanging: true, isRefreshing: true })
    expect(trigger().disabled).toBe(true)
    expect(refresh.disabled).toBe(true)
  })
})
