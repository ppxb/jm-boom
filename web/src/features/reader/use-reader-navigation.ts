import { useCallback, useEffect, useState } from 'react'

type NavigationState = {
  comicId: string
  initialPageIndex: number
  currentIndex: number
}

export function useReaderNavigation({
  comicId,
  initialIndex,
  pageCount,
  pageStep = 1
}: {
  comicId: string
  initialIndex: number
  pageCount: number
  pageStep?: number
}) {
  const initialPageIndex = normalizePageIndex(initialIndex)
  const normalizedPageStep = normalizePageStep(pageStep)
  const [navigationState, setNavigationState] = useState<NavigationState>(() =>
    createNavigationState(comicId, initialPageIndex)
  )
  const [navigationRequestId, setNavigationRequestId] = useState(0)
  const isCurrentNavigationScope =
    navigationState.comicId === comicId && navigationState.initialPageIndex === initialPageIndex
  const currentIndex = isCurrentNavigationScope ? navigationState.currentIndex : initialPageIndex
  const clampPageIndex = useCallback(
    (index: number) => Math.min(Math.max(index, 0), Math.max(pageCount - 1, 0)),
    [pageCount]
  )
  const effectiveCurrentIndex = pageCount > 0 ? clampPageIndex(currentIndex) : currentIndex
  const requestNavigation = useCallback(
    (nextIndex: number) => {
      setNavigationState({
        comicId,
        initialPageIndex,
        currentIndex: nextIndex
      })
      setNavigationRequestId(id => id + 1)
    },
    [comicId, initialPageIndex]
  )
  const goToPreviousPage = useCallback(() => {
    if (pageCount === 0) {
      return
    }

    requestNavigation(clampPageIndex(effectiveCurrentIndex - normalizedPageStep))
  }, [clampPageIndex, effectiveCurrentIndex, normalizedPageStep, pageCount, requestNavigation])
  const goToNextPage = useCallback(() => {
    if (pageCount === 0) {
      return
    }

    requestNavigation(clampPageIndex(effectiveCurrentIndex + normalizedPageStep))
  }, [clampPageIndex, effectiveCurrentIndex, normalizedPageStep, pageCount, requestNavigation])
  const goToPage = useCallback(
    (index: number) => {
      if (pageCount === 0) {
        return
      }

      requestNavigation(clampPageIndex(index))
    },
    [clampPageIndex, pageCount, requestNavigation]
  )
  const setObservedPage = useCallback(
    (index: number) => {
      if (pageCount === 0) {
        return
      }

      setNavigationState(current => {
        const nextIndex = clampPageIndex(index)
        const nextState = {
          comicId,
          initialPageIndex,
          currentIndex: nextIndex
        }

        return isSameNavigationState(current, nextState) ? current : nextState
      })
    },
    [clampPageIndex, comicId, initialPageIndex, pageCount]
  )

  useEffect(() => {
    setNavigationState(current => {
      const nextState = createNavigationState(comicId, initialPageIndex)

      return isSameNavigationState(current, nextState) ? current : nextState
    })
  }, [comicId, initialPageIndex])

  useEffect(() => {
    if (currentIndex < pageCount || pageCount === 0) {
      return
    }

    setNavigationState(current => {
      const nextState = {
        comicId,
        initialPageIndex,
        currentIndex: Math.max(0, pageCount - 1)
      }

      return isSameNavigationState(current, nextState) ? current : nextState
    })
  }, [comicId, currentIndex, initialPageIndex, pageCount])

  return {
    currentIndex,
    effectiveCurrentIndex,
    navigationRequestId,
    isLastPage: pageCount > 0 && effectiveCurrentIndex >= pageCount - normalizedPageStep,
    goToPreviousPage,
    goToNextPage,
    goToPage,
    setObservedPage
  }
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

function createNavigationState(comicId: string, initialPageIndex: number): NavigationState {
  return {
    comicId,
    initialPageIndex,
    currentIndex: initialPageIndex
  }
}

function isSameNavigationState(left: NavigationState, right: NavigationState) {
  return (
    left.comicId === right.comicId &&
    left.initialPageIndex === right.initialPageIndex &&
    left.currentIndex === right.currentIndex
  )
}
