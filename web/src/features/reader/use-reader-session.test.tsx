import { afterEach, beforeEach, describe, expect, vi, test } from 'vitest'
import {
  defaultScheduler,
  notifyManager,
  QueryClient,
  QueryClientProvider
} from '@tanstack/react-query'
import { Window } from 'happy-dom'
import { act, StrictMode, useLayoutEffect } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import type { ComicReadManifestResult } from '@/lib/api/reader'
import { queryKeys } from '@/lib/query-keys'
import { Route } from '@/routes/reader/$comicId'
import * as readerPage from './reader-page'
import type { ReaderChapterItem, ReaderSearch } from './types'
import { useReaderSession } from './use-reader-session'

const ReaderRoute = Route.options.component!
type Session = ReturnType<typeof useReaderSession>
type Request = { path: string; resolve: (response: Response) => void }

let dom: Window
let root: Root
let queryClient: QueryClient
let globals: Map<string, PropertyDescriptor | undefined>
let comicId: string
let search: ReaderSearch
let pageStep: number
let paged: boolean
let session: Session
let requests: Request[]
let chapters: ReaderChapterItem[]
let onLocationChange: ((readId: string, index: number) => void) | undefined

function SessionProbe(props: Parameters<typeof readerPage.ReaderPage>[0]) {
  const current = useReaderSession({
    comicId: props.comicId,
    initialIndex: props.search.page ? props.search.page - 1 : 0,
    pageStep,
    paged,
    chapters,
    onLocationChange
  })
  useLayoutEffect(() => {
    session = current
  })
  return <output>{current.currentIndex}</output>
}

beforeEach(() => {
  dom = new Window({ url: 'http://localhost/reader/101' })
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
  notifyManager.setScheduler(queueMicrotask)
  notifyManager.setNotifyFunction(callback => {
    void act(callback)
  })
  comicId = '101'
  search = { albumId: '100' }
  pageStep = 1
  paged = false
  requests = []
  chapters = []
  onLocationChange = undefined
  vi.spyOn(Route, 'useParams').mockReturnValue({ comicId })
  vi.spyOn(Route, 'useSearch').mockReturnValue(search)
  // Keep the real route and session hook, avoiding unrelated reader UI and side effects.
  vi.spyOn(readerPage, 'ReaderPage').mockImplementation(SessionProbe)
  const mockFetch = (input: Parameters<typeof fetch>[0]) =>
    new Promise<Response>(resolve => requests.push({ path: String(input), resolve }))
  vi.spyOn(globalThis, 'fetch').mockImplementation(mockFetch)
})

afterEach(async () => {
  await act(async () => {
    root.unmount()
    for (const request of requests) request.resolve(unavailable())
  })
  queryClient.clear()
  await dom.happyDOM.close()
  notifyManager.setNotifyFunction(callback => callback())
  notifyManager.setScheduler(defaultScheduler)
  vi.restoreAllMocks()
  for (const [key, descriptor] of globals) {
    if (descriptor) Object.defineProperty(globalThis, key, descriptor)
    else Reflect.deleteProperty(globalThis, key)
  }
})

async function render(nextComicId = comicId, nextSearch = search, nextPageStep = pageStep) {
  comicId = nextComicId
  search = nextSearch
  pageStep = nextPageStep
  vi.spyOn(Route, 'useParams').mockReturnValue({ comicId })
  vi.spyOn(Route, 'useSearch').mockReturnValue(search)
  await act(async () => {
    root.render(
      <QueryClientProvider client={queryClient}>
        <StrictMode>
          <ReaderRoute />
        </StrictMode>
      </QueryClientProvider>
    )
  })
}

function manifest(readId: string, pageCount: number): ComicReadManifestResult {
  return {
    readId,
    pageCount,
    pages: Array.from({ length: pageCount }, (_, index) => ({
      index,
      name: `${index}.webp`,
      path: `/api/reader/${readId}/pages/${index}`
    }))
  }
}

function seed(readId: string, pageCount: number) {
  queryClient.setQueryData(queryKeys.readerManifest(readId), manifest(readId, pageCount))
}

async function respond(
  request: Request,
  readId: string,
  pageCount: number,
  settled?: Promise<void>
) {
  const data = manifest(readId, pageCount)
  await act(async () => {
    request.resolve(
      Response.json({
        chapter_id: readId,
        pages: data.pages.map(page => ({ index: page.index, name: page.name, url: page.path }))
      })
    )
    await settled
  })
}

function unavailable() {
  return Response.json({ error: 'Unavailable', retryable: true }, { status: 503 })
}

describe('reader route session lifecycle', () => {
  test('preserves the requested page while loading and clamps it when the manifest arrives', async () => {
    await render('101', { albumId: '100', page: 50 })
    expect(session.currentIndex).toBe(49)
    expect(session.pageCount).toBe(0)
    expect(session.pageWindow).toEqual([])
    expect(session.isManifestLoading).toBe(true)
    await act(async () => {
      session.goToNextPage()
      session.observePage(3)
    })
    expect(session.currentIndex).toBe(49)
    expect(session.navigationCommand).toEqual({ id: 0, targetIndex: 49 })
    await respond(requests[0], '101', 20)
    expect(session.currentIndex).toBe(19)
    expect(session.pageSrc).toBe('/api/reader/101/pages/19')
    expect(session.isLastPage).toBe(true)
    expect(session.isManifestLoading).toBe(false)
  })

  test('ordinary page turns and double-page mode changes keep the same mounted session', async () => {
    seed('101', 10)
    await render('101', { albumId: '100', page: 4 })
    const element = document.querySelector('output')
    expect(session.currentIndex).toBe(3)
    await act(async () => session.goToNextPage())
    expect(session.currentIndex).toBe(4)
    await render('101', search, 2)
    expect(document.querySelector('output')).toBe(element)
    expect(session.currentIndex).toBe(4)
    expect(session.pageWindow.map(page => page.index)).toEqual([2, 3, 4, 5, 6, 7])
    await act(async () => session.goToNextPage())
    expect(session.currentIndex).toBe(6)
    await act(async () => session.goToPreviousPage())
    expect(session.currentIndex).toBe(4)
    expect(session.navigationCommand.id).toBe(3)
    expect(requests).toHaveLength(0)
  })

  test('changing chapters keeps the session mounted and issues a navigation command', async () => {
    seed('101', 10)
    seed('201', 8)
    await render()
    const element = document.querySelector('output')
    await act(async () => session.goToPage(7))
    expect(session.navigationCommand.id).toBe(1)
    await render('201', { albumId: '100', page: 1 })
    expect(document.querySelector('output')).toBe(element)
    expect(session.currentIndex).toBe(0)
    expect(session.pageSrc).toBe('/api/reader/201/pages/0')
    expect(session.navigationCommand).toEqual({ id: 2, targetIndex: 0, targetKey: '201:0' })
    await render('101', { albumId: '100', page: 1 })
    expect(session.currentIndex).toBe(0)
    expect(requests).toHaveLength(0)
  })

  test('route page jumps keep the same mounted reader and equivalent searches do not reset progress', async () => {
    seed('101', 20)
    await render()
    const first = document.querySelector('output')
    await act(async () => session.goToPage(5))
    await render('101', { albumId: '100', page: 1 })
    expect(document.querySelector('output')).toBe(first)
    expect(session.currentIndex).toBe(5)
    await render('101', { albumId: '100', page: 8 })
    expect(document.querySelector('output')).toBe(first)
    expect(session.currentIndex).toBe(7)
    expect(session.navigationCommand).toEqual({ id: 2, targetIndex: 7, targetKey: '101:7' })
  })

  test('a late response from the previous chapter cannot reset the new session', async () => {
    await render('101', { albumId: '100', page: 6 })
    const previous = requests[0]
    await render('201', { albumId: '100', page: 1 })
    expect(requests).toHaveLength(2)
    await respond(requests[1], '201', 4)
    await act(async () => session.goToPage(2))
    await respond(previous, '101', 20)
    expect(session.currentIndex).toBe(2)
    expect(session.pageCount).toBe(4)
    expect(session.pageSrc).toBe('/api/reader/201/pages/2')
    expect(session.navigationCommand).toEqual({ id: 2, targetIndex: 2, targetKey: '201:2' })
  })

  test('scroll observations do not issue commands, while repeated jumps still do', async () => {
    seed('101', 20)
    await render()
    const initialCommand = session.navigationCommand
    await act(async () => session.observePage(5))
    await act(async () => session.observePage(5))
    expect(session.currentIndex).toBe(5)
    expect(session.navigationCommand).toBe(initialCommand)
    await act(async () => session.goToPage(5))
    expect(session.navigationCommand).toEqual({ id: 1, targetIndex: 5, targetKey: '101:5' })
    await act(async () => session.goToPage(5))
    expect(session.navigationCommand).toEqual({ id: 2, targetIndex: 5, targetKey: '101:5' })
    await act(async () => session.goToPage(999))
    expect(session.currentIndex).toBe(19)
    await act(async () => session.goToPage(-1))
    expect(session.currentIndex).toBe(0)
  })

  test('a smaller manifest clamps stored progress without restoring it when pages are added', async () => {
    seed('101', 12)
    await render('101', { albumId: '100', page: 12 })
    expect(session.currentIndex).toBe(11)
    await act(async () => seed('101', 4))
    expect(session.currentIndex).toBe(3)
    await act(async () => seed('101', 12))
    expect(session.currentIndex).toBe(3)
    expect(session.pageSrc).toBe('/api/reader/101/pages/3')
    expect(session.navigationCommand).toEqual({ id: 0, targetIndex: 11 })
  })

  test('manifest errors and retry preserve the requested starting page', async () => {
    await render('101', { albumId: '100', page: 4 })
    await act(async () => requests[0].resolve(unavailable()))
    expect(session.manifestError).not.toBeNull()
    expect(session.currentIndex).toBe(3)
    await act(async () => {
      void session.retry()
    })
    expect(requests).toHaveLength(2)
    await respond(requests[1], '101', 6)
    expect(session.manifestError).toBeNull()
    expect(session.currentIndex).toBe(3)
    expect(session.pageSrc).toBe('/api/reader/101/pages/3')
  })

  test('turns across chapters in both directions without remounting or mixing local page numbers', async () => {
    chapters = ['101', '201', '301'].map(id => ({ id, title: id }))
    seed('101', 3)
    seed('201', 5)
    seed('301', 4)
    await render()
    const element = document.querySelector('output')
    await act(async () => session.goToPage(2))
    await act(async () => session.goToNextPage())
    expect(document.querySelector('output')).toBe(element)
    expect(session.activeReadId).toBe('201')
    expect(session.currentIndex).toBe(0)
    expect(session.pageCount).toBe(5)
    expect(session.pageSrc).toBe('/api/reader/201/pages/0')
    await act(async () => session.goToPreviousPage())
    expect(session.activeReadId).toBe('101')
    expect(session.currentIndex).toBe(2)
    expect(session.pageCount).toBe(3)
  })

  test('appending a prefetched chapter does not change progress until its page is observed', async () => {
    chapters = ['101', '201'].map(id => ({ id, title: id }))
    seed('101', 3)
    await render()
    await act(async () => requests[0].resolve(unavailable()))
    expect(session.nextBoundary.error).not.toBeNull()
    expect(session.activeReadId).toBe('101')
    await act(async () => session.loadBoundary(1))
    await respond(requests[1], '201', 4)
    expect(session.pages).toHaveLength(7)
    expect(session.currentIndex).toBe(0)
    expect(session.activeReadId).toBe('101')
    const command = session.navigationCommand
    await act(async () => session.observePage(4))
    expect(session.activeReadId).toBe('201')
    expect(session.currentIndex).toBe(1)
    expect(session.pageCount).toBe(4)
    expect(session.navigationCommand).toBe(command)
  })

  test('retains viewport chapters while dropping chapters far behind continuous reading', async () => {
    chapters = ['101', '201', '301', '401', '501'].map(id => ({ id, title: id }))
    chapters.forEach(chapter => seed(chapter.id, 3))
    await render()
    await act(async () => {
      session.observeVisibleRange(2, 4)
      session.observePage(3)
    })
    expect(session.pages.some(page => page.chapterId === '101')).toBe(true)
    let index = session.pages.findIndex(page => page.chapterId === '301')
    await act(async () => {
      session.observeVisibleRange(index, index + 2)
      session.observePage(index)
    })
    expect([...new Set(session.pages.map(page => page.chapterId))]).toEqual(['201', '301', '401'])
    expect(session.currentIndex).toBe(0)
    index = session.pages.findIndex(page => page.chapterId === '401')
    await act(async () => {
      session.observeVisibleRange(index, index + 2)
      session.observePage(index)
    })
    expect([...new Set(session.pages.map(page => page.chapterId))]).toEqual(['301', '401', '501'])
  })

  test('failed explicit jumps keep the old page and can be retried', async () => {
    seed('101', 6)
    await render('101', { albumId: '100', page: 4 })
    await act(async () => {
      void session.goToChapter('201', -1)
    })
    expect(session.pageSrc).toBe('/api/reader/101/pages/3')
    await act(async () => requests[0].resolve(unavailable()))
    expect(session.jump?.error).not.toBeNull()
    expect(session.activeReadId).toBe('101')
    await act(async () => {
      void session.retry()
    })
    await respond(requests[1], '201', 8)
    expect(session.jump).toBeNull()
    expect(session.activeReadId).toBe('201')
    expect(session.currentIndex).toBe(7)
  })

  test('a late directory jump cannot replace a newer successful jump', async () => {
    seed('101', 6)
    await render()
    await act(async () => {
      void session.goToChapter('201')
    })
    await act(async () => {
      void session.goToChapter('301', 2)
    })
    await respond(requests[1], '301', 8)
    await respond(requests[0], '201', 8)
    expect(session.activeReadId).toBe('301')
    expect(session.currentIndex).toBe(2)
    expect(session.jump).toBeNull()
  })

  test('URL acknowledgements do not navigate again, while external page requests do', async () => {
    seed('101', 6)
    const publish = vi.fn()
    onLocationChange = publish
    await render()
    await act(async () => session.goToPage(3))
    expect(publish).toHaveBeenLastCalledWith('101', 3)
    const command = session.navigationCommand
    await render('101', { albumId: '100', page: 4 })
    expect(session.navigationCommand).toBe(command)
    await render('101', { albumId: '100', page: 2 })
    expect(session.currentIndex).toBe(1)
    expect(session.navigationCommand.id).toBe(command.id + 1)
  })

  test('double page mode leaves an odd final page alone and never pairs across chapters', async () => {
    chapters = ['101', '201'].map(id => ({ id, title: id }))
    seed('101', 3)
    seed('201', 4)
    await render('101', { albumId: '100', page: 3 }, 2)
    expect(session.currentIndex).toBe(2)
    expect(session.visiblePageCount).toBe(1)
    await act(async () => session.goToNextPage())
    expect(session.activeReadId).toBe('201')
    expect(session.currentIndex).toBe(0)
    expect(session.visiblePageCount).toBe(2)
    await act(async () => session.goToPreviousPage())
    expect(session.activeReadId).toBe('101')
    expect(session.currentIndex).toBe(2)
  })

  test('coalesced URL acknowledgements do not swallow later external requests', async () => {
    seed('101', 8)
    onLocationChange = vi.fn()
    await render()
    await act(async () => session.goToPage(3))
    await act(async () => session.goToPage(4))
    await render('101', { albumId: '100', page: 5 })
    await render('101', { albumId: '100', page: 4 })
    expect(session.currentIndex).toBe(3)
  })

  test('ordered acknowledgements preserve newer positions without issuing scroll commands', async () => {
    seed('101', 8)
    onLocationChange = vi.fn()
    await render()
    await act(async () => session.goToPage(3))
    await act(async () => session.goToPage(4))
    const firstCommand = session.navigationCommand
    await render('101', { albumId: '100', page: 4 })
    expect(session.currentIndex).toBe(4)
    expect(session.navigationCommand).toBe(firstCommand)
    await act(async () => session.goToPage(5))
    const newestCommand = session.navigationCommand
    await render('101', { albumId: '100', page: 5 })
    expect(session.currentIndex).toBe(5)
    expect(session.navigationCommand).toBe(newestCommand)
    await render('101', { albumId: '100', page: 6 })
    expect(session.navigationCommand).toBe(newestCommand)
  })

  test('different books get isolated sessions before their manifests arrive', async () => {
    seed('101', 8)
    await render('101', { albumId: '100', page: 5 })
    const previous = document.querySelector('output')
    await render('201', { albumId: '200', page: 2 })
    expect(document.querySelector('output')).not.toBe(previous)
    expect(session.activeReadId).toBe('201')
    expect(session.pageCount).toBe(0)
    expect(session.pageSrc).toBe('')
    await act(async () => requests[0].resolve(unavailable()))
    expect(session.manifestError).not.toBeNull()
    expect(session.activeReadId).toBe('201')
  })

  test('a mounted image failure revokes readiness independently of cached dimensions', async () => {
    seed('101', 3)
    await render()
    const page = session.pages[0]
    await act(async () => {
      session.onPageLoaded(page, { naturalWidth: 800, naturalHeight: 1200 } as HTMLImageElement)
      session.onPageReady(page.key, true)
    })
    expect(session.isPageReady).toBe(true)
    await act(async () => session.onPageReady(page.key, false))
    expect(session.isPageReady).toBe(false)
    expect(session.pages[0].width).toBe(800)
  })

  test('paged chapter transitions retain the source page until the next turn', async () => {
    paged = true
    chapters = ['101', '201'].map(id => ({ id, title: id }))
    seed('101', 3)
    seed('201', 4)
    await render('101', { albumId: '100', page: 3 })
    const command = session.navigationCommand
    await act(async () => session.goToNextPage())
    expect(session.transition?.currentChapter.id).toBe('101')
    expect(session.transition?.boundary.chapter?.id).toBe('201')
    expect(session.activeReadId).toBe('101')
    expect(session.currentIndex).toBe(2)
    expect(session.navigationCommand).toBe(command)
    await act(async () => session.goToNextPage())
    expect(session.transition).toBeNull()
    expect(session.activeReadId).toBe('201')
    expect(session.currentIndex).toBe(0)
    await act(async () => session.goToPreviousPage())
    expect(session.transition?.direction).toBe(-1)
    expect(session.activeReadId).toBe('201')
    await act(async () => session.goToNextPage())
    expect(session.transition).toBeNull()
    expect(session.activeReadId).toBe('201')
    await act(async () => session.goToPreviousPage())
    await act(async () => session.goToPreviousPage())
    expect(session.activeReadId).toBe('101')
    expect(session.currentIndex).toBe(2)
  })

  test('cancelling a loading transition keeps the original page when the request finishes', async () => {
    paged = true
    chapters = ['101', '201'].map(id => ({ id, title: id }))
    seed('101', 3)
    await render('101', { albumId: '100', page: 3 })
    await act(async () => session.goToNextPage())
    await act(async () => session.goToNextPage())
    expect(session.transition?.boundary.loading).toBe(true)
    await act(async () => session.goToPreviousPage())
    await respond(requests[0], '201', 4)
    expect(session.transition).toBeNull()
    expect(session.jump).toBeNull()
    expect(session.activeReadId).toBe('101')
    expect(session.currentIndex).toBe(2)
  })

  test('a failed transition retries its destination rather than the current chapter', async () => {
    paged = true
    chapters = ['101', '201'].map(id => ({ id, title: id }))
    seed('101', 3)
    await render('101', { albumId: '100', page: 3 })
    await act(async () => requests[0].resolve(unavailable()))
    await act(async () => session.goToNextPage())
    expect(session.transition?.boundary.error).not.toBeNull()
    let retryTask: Promise<void> | undefined
    await act(async () => {
      retryTask = session.retry()
    })
    expect(requests).toHaveLength(2)
    expect(requests[1].path).toContain('/reader/201/manifest')
    await respond(requests[1], '201', 4, retryTask)
    expect(session.transition).toBeNull()
    expect(session.activeReadId).toBe('201')
  })

  test('first and last chapter transitions do not navigate beyond the book', async () => {
    paged = true
    seed('101', 1)
    await render()
    const command = session.navigationCommand
    await act(async () => session.goToNextPage())
    expect(session.transition?.boundary.chapter).toBeNull()
    await act(async () => session.goToNextPage())
    expect(session.navigationCommand).toBe(command)
    await act(async () => session.goToPreviousPage())
    expect(session.transition).toBeNull()
    await act(async () => session.goToPreviousPage())
    expect(session.transition?.direction).toBe(-1)
    expect(session.transition?.boundary.chapter).toBeNull()
    await act(async () => session.goToNextPage())
    expect(session.transition).toBeNull()
  })

  test('a new transition does not inherit an unrelated failed directory jump', async () => {
    paged = true
    chapters = ['101', '201', '301'].map(id => ({ id, title: id }))
    seed('101', 3)
    seed('201', 4)
    await render('101', { albumId: '100', page: 3 })
    await act(async () => {
      void session.goToChapter('301')
    })
    await act(async () => requests[0].resolve(unavailable()))
    expect(session.jump?.error).not.toBeNull()
    await act(async () => session.goToNextPage())
    expect(session.transition?.boundary.chapter?.id).toBe('201')
    expect(session.transition?.boundary.error).toBeNull()
    expect(session.jump).toBeNull()
  })

  test('explicit page jumps and switching to strip dismiss the paged transition', async () => {
    paged = true
    seed('101', 3)
    await render('101', { albumId: '100', page: 3 })
    await act(async () => session.goToNextPage())
    await act(async () => session.goToPage(0))
    expect(session.transition).toBeNull()
    await act(async () => session.goToPreviousPage())
    paged = false
    await render()
    expect(session.transition).toBeNull()
    expect(session.currentIndex).toBe(0)
  })
})
