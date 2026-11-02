import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest'
import { Window } from 'happy-dom'
import { act, StrictMode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import { ReaderPageImage } from './reader-image'

const CACHE_LIMIT = 1000
let sequence = 0
let prefix: string
let dom: Window
let root: Root
let globals: Map<string, PropertyDescriptor | undefined>

beforeEach(() => {
  prefix = `/api/reader/image-size-test-${sequence++}`
  dom = new Window({ url: 'http://localhost' })
  const values = {
    window: dom,
    document: dom.document,
    navigator: dom.navigator,
    Event: dom.Event,
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
})

afterEach(async () => {
  await act(async () => root.unmount())
  await dom.happyDOM.close()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

async function render(sources: string[]) {
  await act(async () => {
    root.render(
      <StrictMode>
        {sources.map((src, index) => (
          <ReaderPageImage
            key={index}
            src={src}
            label={`第 ${index + 1} 页`}
            fitContainer
            placeholderClassName="min-h-[64vh]"
            showLoadingIndicator={sources.length === 1}
          />
        ))}
      </StrictMode>
    )
  })
}

function images() {
  return Array.from(document.querySelectorAll('img'))
}

async function load(targets = images(), width = 800, height = 1200) {
  await act(async () => {
    for (const image of targets) {
      Object.defineProperties(image, {
        naturalWidth: { value: width, configurable: true },
        naturalHeight: { value: height, configurable: true }
      })
      image.dispatchEvent(new Event('load'))
    }
  })
}

async function fillCache(count = CACHE_LIMIT) {
  const sources = Array.from({ length: count }, (_, index) => `${prefix}/${index}.webp`)
  await render(sources)
  await load()
  return sources
}

function expectSize(image: HTMLImageElement, ratio = '800 / 1200') {
  expect(image.parentElement!.style.aspectRatio).toBe(ratio)
  expect(image.parentElement!.classList.contains('min-h-[64vh]')).toBe(false)
  expect(image.classList.contains('opacity-100')).toBe(true)
}

function expectUnknownSize(image: HTMLImageElement) {
  expect(image.parentElement!.style.aspectRatio).toBe('')
  expect(image.parentElement!.classList.contains('min-h-[64vh]')).toBe(true)
  expect(image.classList.contains('opacity-0')).toBe(true)
}

describe('reader image size cache', () => {
  test('reports an image error and unmount even when dimensions were already cached', async () => {
    const src = `${prefix}/ready.webp`
    await render([src])
    await load()
    await render([])
    const ready = vi.fn()
    await act(async () =>
      root.render(<ReaderPageImage src={src} label="第 1 页" onReadyChange={ready} />)
    )
    await load()
    expect(ready).toHaveBeenLastCalledWith(true)
    await act(async () => images()[0].dispatchEvent(new Event('error')))
    expect(ready).toHaveBeenLastCalledWith(false)
    await load()
    await render([])
    expect(ready).toHaveBeenLastCalledWith(false)
  })
  test('reserves persisted dimensions before loading and prefers the actual decoded size', async () => {
    await act(async () =>
      root.render(
        <ReaderPageImage
          src={`${prefix}/persisted.webp`}
          label="第 1 页"
          width={600}
          height={1800}
          fitContainer
          placeholderClassName="min-h-[64vh]"
        />
      )
    )
    const image = images()[0]
    expect(image.parentElement!.style.aspectRatio).toBe('600 / 1800')
    expect(image.parentElement!.classList.contains('min-h-[64vh]')).toBe(false)
    expect(image.classList.contains('opacity-0')).toBe(true)
    await load([image], 800, 1200)
    expectSize(image)
  })
  test('restores the exact aspect ratio on remount without waiting for another load event', async () => {
    const src = `${prefix}/page.webp`
    await render([src])
    expectUnknownSize(images()[0])
    await load()
    await render([])
    await render([src])
    expectSize(images()[0])
    expect(document.body.textContent).not.toContain('正在加载')
  })

  test('retains at most the latest 1000 distinct successful image sizes', async () => {
    const sources = await fillCache(CACHE_LIMIT + 1)
    await render([])
    await render([sources[0], sources[1], sources[CACHE_LIMIT]])
    expectUnknownSize(images()[0])
    expectSize(images()[1])
    expectSize(images()[2])
  })

  test('refreshes recency when a cached image is revisited without a new load event', async () => {
    const sources = await fillCache()
    await render([])
    await render([sources[0]])
    expectSize(images()[0])
    await render([`${prefix}/next.webp`])
    await load()
    await render([sources[0], sources[1]])
    expectSize(images()[0])
    expectUnknownSize(images()[1])
  })

  test('updates existing dimensions without consuming an extra cache entry', async () => {
    const sources = await fillCache()
    await render([sources[0]])
    await load(images(), 1200, 800)
    await load(images(), 1200, 800)
    await render([`${prefix}/next.webp`])
    await load()
    await render([sources[0], sources[1], sources[2]])
    expectSize(images()[0], '1200 / 800')
    expectUnknownSize(images()[1])
    expectSize(images()[2])
  })

  test('keeps a mounted image stable after its shared size record is evicted', async () => {
    const src = `${prefix}/visible.webp`
    await render([src])
    await load()
    const visible = images()[0]
    const otherSources = Array.from(
      { length: CACHE_LIMIT },
      (_, index) => `${prefix}/${index}.webp`
    )
    await render([src, ...otherSources])
    await load(images().slice(1))
    expect(images()[0]).toBe(visible)
    expectSize(visible)
    await render([])
    await render([src])
    expectUnknownSize(images()[0])
  })

  test('does not cache invalid dimensions or evict valid records for them', async () => {
    const sources = await fillCache()
    const invalid = `${prefix}/invalid.webp`
    await render([invalid])
    await load(images(), 0, 1200)
    await render([])
    await render([sources[0], invalid])
    expectSize(images()[0])
    expectUnknownSize(images()[1])
  })
})
