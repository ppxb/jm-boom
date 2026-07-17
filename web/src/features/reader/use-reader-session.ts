import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import type { ComicReadManifestPage } from '@/lib/api/reader'
import type { ReaderWindowPage } from './types'
import { useReaderManifestQuery } from './use-reader-manifest-query'

const EMPTY_MANIFEST_PAGES: ComicReadManifestPage[] = []

type ReaderPosition = {
  scope: string
  currentIndex: number
}

export type ReaderNavigationCommand = {
  id: number
  targetIndex: number
}

export function useReaderSession({
  comicId,
  initialIndex = 0,
  pageStep = 1
}: {
  comicId: string
  initialIndex?: number
  pageStep?: number
}) {
  const manifest = useReaderManifestQuery(comicId)
  const pages = manifest.data?.pages ?? EMPTY_MANIFEST_PAGES
  const pageCount = pages.length
  const initialPageIndex = normalizePageIndex(initialIndex)
  const normalizedPageStep = normalizePageStep(pageStep)
  const scope = `${comicId}:${initialPageIndex}`
  const scopeRef = useRef(scope)
  const [position, setPosition] = useState<ReaderPosition>(() => ({
    scope,
    currentIndex: initialPageIndex
  }))
  const [navigationCommand, setNavigationCommand] = useState<ReaderNavigationCommand>(() => ({
    id: 0,
    targetIndex: initialPageIndex
  }))
  const scopedCurrentIndex = position.scope === scope ? position.currentIndex : initialPageIndex
  const clampPageIndex = useCallback(
    (index: number) => Math.min(Math.max(index, 0), Math.max(pageCount - 1, 0)),
    [pageCount]
  )
  const currentIndex = pageCount > 0 ? clampPageIndex(scopedCurrentIndex) : scopedCurrentIndex
  const pageSrc = pages[currentIndex]?.path ?? ''
  const pageWindow = useMemo(
    () => createReaderWindow(pages, currentIndex, normalizedPageStep),
    [currentIndex, normalizedPageStep, pages]
  )

  const requestNavigation = useCallback(
    (index: number) => {
      if (pageCount <= 0) {
        return
      }

      const targetIndex = clampPageIndex(index)
      setPosition({ scope, currentIndex: targetIndex })
      setNavigationCommand(current => ({ id: current.id + 1, targetIndex }))
    },
    [clampPageIndex, pageCount, scope]
  )
  const goToPreviousPage = useCallback(
    () => requestNavigation(currentIndex - normalizedPageStep),
    [currentIndex, normalizedPageStep, requestNavigation]
  )
  const goToNextPage = useCallback(
    () => requestNavigation(currentIndex + normalizedPageStep),
    [currentIndex, normalizedPageStep, requestNavigation]
  )
  const goToPage = useCallback((index: number) => requestNavigation(index), [requestNavigation])
  const observePage = useCallback(
    (index: number) => {
      if (pageCount <= 0) {
        return
      }

      const nextIndex = clampPageIndex(index)
      setPosition(current => {
        const nextPosition = { scope, currentIndex: nextIndex }

        return isSamePosition(current, nextPosition) ? current : nextPosition
      })
    },
    [clampPageIndex, pageCount, scope]
  )

  useEffect(() => {
    if (scopeRef.current === scope) {
      return
    }

    scopeRef.current = scope
    setPosition(current => {
      const nextPosition = { scope, currentIndex: initialPageIndex }

      return isSamePosition(current, nextPosition) ? current : nextPosition
    })
    setNavigationCommand(current => ({
      id: current.id + 1,
      targetIndex: initialPageIndex
    }))
  }, [initialPageIndex, scope])

  useEffect(() => {
    if (scopedCurrentIndex < pageCount || pageCount <= 0) {
      return
    }

    setPosition({ scope, currentIndex: Math.max(0, pageCount - 1) })
  }, [pageCount, scope, scopedCurrentIndex])

  return {
    currentIndex,
    pageCount,
    pages,
    pageSrc,
    pageWindow,
    navigationCommand,
    isLastPage: pageCount > 0 && currentIndex >= pageCount - normalizedPageStep,
    isManifestLoading: manifest.isLoading,
    manifestError: manifest.isError ? manifest.error : null,
    isFetching: manifest.isFetching,
    goToPreviousPage,
    goToNextPage,
    goToPage,
    observePage,
    retry: () => void manifest.refetch()
  }
}

function createReaderWindow(
  pages: ComicReadManifestPage[],
  currentIndex: number,
  pageStep: number
): ReaderWindowPage[] {
  if (pages.length === 0) {
    return []
  }

  const start = Math.max(0, currentIndex - pageStep)
  const end = Math.min(pages.length - 1, currentIndex + pageStep * 2 - 1)
  const window: ReaderWindowPage[] = []

  for (let index = start; index <= end; index += 1) {
    const page = pages[index]

    if (page) {
      window.push({ index, src: page.path })
    }
  }

  return window
}

function normalizePageIndex(index: number) {
  if (!Number.isFinite(index)) {
    return 0
  }

  return Math.max(0, Math.floor(index))
}

function normalizePageStep(pageStep: number) {
  if (!Number.isFinite(pageStep)) {
    return 1
  }

  return Math.max(1, Math.floor(pageStep))
}

function isSamePosition(left: ReaderPosition, right: ReaderPosition) {
  return left.scope === right.scope && left.currentIndex === right.currentIndex
}
