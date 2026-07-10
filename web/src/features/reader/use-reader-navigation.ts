import { useCallback, useEffect, useState } from 'react'

type NavigationState = {
  comicId: string
  endpoint: string
  initialPageIndex: number
  currentIndex: number
}

export function useReaderNavigation({
  comicId,
  endpoint,
  initialIndex,
  pageCount,
  pageStep = 1
}: {
  comicId: string
  endpoint: string
  initialIndex: number
  pageCount: number
  pageStep?: number
}) {
  const initialPageIndex = normalizePageIndex(initialIndex)
  const normalizedPageStep = normalizePageStep(pageStep)
  const [navigationState, setNavigationState] = useState<NavigationState>(() =>
    createNavigationState(comicId, endpoint, initialPageIndex)
  )
  const [navigationRequestId, setNavigationRequestId] = useState(0)
  const isCurrentNavigationScope =
    navigationState.comicId === comicId &&
    navigationState.endpoint === endpoint &&
    navigationState.initialPageIndex === initialPageIndex
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
        endpoint,
        initialPageIndex,
        currentIndex: nextIndex
      })
      setNavigationRequestId(id => id + 1)
    },
    [comicId, endpoint, initialPageIndex]
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
          endpoint,
          initialPageIndex,
          currentIndex: nextIndex
        }

        return isSameNavigationState(current, nextState) ? current : nextState
      })
    },
    [clampPageIndex, comicId, endpoint, initialPageIndex, pageCount]
  )

  useEffect(() => {
    setNavigationState(current => {
      const nextState = createNavigationState(comicId, endpoint, initialPageIndex)

      return isSameNavigationState(current, nextState) ? current : nextState
    })
  }, [comicId, endpoint, initialPageIndex])

  useEffect(() => {
    if (currentIndex < pageCount || pageCount === 0) {
      return
    }

    setNavigationState(current => {
      const nextState = {
        comicId,
        endpoint,
        initialPageIndex,
        currentIndex: Math.max(0, pageCount - 1)
      }

      return isSameNavigationState(current, nextState) ? current : nextState
    })
  }, [comicId, currentIndex, endpoint, initialPageIndex, pageCount])

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

function createNavigationState(
  comicId: string,
  endpoint: string,
  initialPageIndex: number
): NavigationState {
  return {
    comicId,
    endpoint,
    initialPageIndex,
    currentIndex: initialPageIndex
  }
}

function isSameNavigationState(left: NavigationState, right: NavigationState) {
  return (
    left.comicId === right.comicId &&
    left.endpoint === right.endpoint &&
    left.initialPageIndex === right.initialPageIndex &&
    left.currentIndex === right.currentIndex
  )
}
