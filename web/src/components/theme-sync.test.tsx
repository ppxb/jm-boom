import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import { initializeTheme } from '@/lib/theme'
import { useThemeStore } from '@/stores/theme-store'
import { ThemeSync } from './theme-sync'

let root: Root
let media: MediaQueryList

beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true)
  localStorage.clear()
  document.documentElement.className = ''
  media = Object.assign(new EventTarget(), {
    matches: false,
    media: '(prefers-color-scheme: dark)',
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn()
  })
  vi.spyOn(window, 'matchMedia').mockReturnValue(media)
  useThemeStore.getState().setTheme('system')
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
})

afterEach(async () => {
  await act(async () => root.unmount())
  document.body.replaceChildren()
  localStorage.clear()
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

test('the inlined boot function restores the existing theme before React mounts', () => {
  localStorage.setItem('theme', 'dark')
  const boot = new Function(`return (${initializeTheme.toString()})()`)
  expect(boot()).toBe('dark')
  expect(document.documentElement.classList.contains('dark')).toBe(true)
  expect(document.documentElement.style.colorScheme).toBe('dark')
  expect(document.documentElement.classList.contains('light')).toBe(false)
})

test('invalid preferences and unavailable storage follow the system', () => {
  Object.assign(media, { matches: true })
  localStorage.setItem('theme', 'invalid')
  expect(initializeTheme()).toBe('system')
  expect(document.documentElement.classList.contains('dark')).toBe(true)
  vi.spyOn(Storage.prototype, 'getItem').mockImplementation(() => {
    throw new Error('denied')
  })
  expect(initializeTheme()).toBe('system')
  vi.spyOn(Storage.prototype, 'setItem').mockImplementation(() => {
    throw new Error('denied')
  })
  expect(() => useThemeStore.getState().setTheme('light')).not.toThrow()
  expect(document.documentElement.style.colorScheme).toBe('light')
})

test('system changes update the resolved theme while a manual choice stays fixed', async () => {
  await act(async () =>
    root.render(
      <StrictMode>
        <ThemeSync />
      </StrictMode>
    )
  )
  Object.assign(media, { matches: true })
  media.dispatchEvent(new Event('change'))
  expect(useThemeStore.getState().resolvedTheme).toBe('dark')
  expect(document.documentElement.style.colorScheme).toBe('dark')
  useThemeStore.getState().setTheme('light')
  media.dispatchEvent(new Event('change'))
  expect(useThemeStore.getState().resolvedTheme).toBe('light')
  expect(localStorage.getItem('theme')).toBe('light')
  useThemeStore.getState().setTheme('system')
  expect(useThemeStore.getState().resolvedTheme).toBe('dark')
})

test('changes from other tabs update the theme and listeners are cleaned up', async () => {
  await act(async () => root.render(<ThemeSync />))
  window.dispatchEvent(new StorageEvent('storage', { key: 'theme', newValue: 'dark' }))
  expect(useThemeStore.getState().theme).toBe('dark')
  expect(document.documentElement.style.colorScheme).toBe('dark')
  window.dispatchEvent(new StorageEvent('storage', { key: 'theme', newValue: null }))
  expect(useThemeStore.getState().theme).toBe('system')
  await act(async () => root.render(null))
  Object.assign(media, { matches: true })
  media.dispatchEvent(new Event('change'))
  window.dispatchEvent(new StorageEvent('storage', { key: 'theme', newValue: 'dark' }))
  expect(useThemeStore.getState().resolvedTheme).toBe('light')
})
