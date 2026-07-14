import { useMemo } from 'react'

import type { ReaderWindowPage } from './types'
import { useReaderManifestQuery } from './use-reader-manifest-query'
import { useReaderNavigation } from './use-reader-navigation'
import { useAdjacentReaderPageQueries, useReaderPageQuery } from './use-reader-page-query'
import { useReaderPrefetch } from './use-reader-prefetch'

const EMPTY_MANIFEST_PAGES: never[] = []

export function useReaderPages(
  comicId: string,
  initialIndex = 0,
  pageStep = 1,
  prefetchEnabled = true
) {
  const manifest = useReaderManifestQuery(comicId)
  const manifestPages = manifest.data?.pages ?? EMPTY_MANIFEST_PAGES
  const pageCount = manifestPages.length
  const {
    effectiveCurrentIndex,
    navigationRequestId,
    isLastPage,
    goToPreviousPage,
    goToNextPage,
    goToPage,
    setObservedPage
  } = useReaderNavigation({
    comicId,
    initialIndex,
    pageCount,
    pageStep
  })
  const { page, pageSrc, isPageReady, pageQueryKey, requestPage } = useReaderPageQuery({
    comicId,
    manifestPages,
    pageIndex: effectiveCurrentIndex,
    enabled: manifest.isSuccess && pageCount > 0
  })
  const adjacentPages = useAdjacentReaderPageQueries({
    pageIndex: effectiveCurrentIndex,
    pageCount,
    pageStep,
    enabled: prefetchEnabled && manifest.isSuccess && pageCount > 0,
    pageQueryKey,
    requestPage
  })
  const pageWindow = useMemo<ReaderWindowPage[]>(() => {
    const pages = [...adjacentPages]

    if (isPageReady && page.data && pageSrc.length > 0) {
      pages.push({ index: page.data.index, src: pageSrc })
    }

    pages.sort((left, right) => left.index - right.index)

    return pages
  }, [adjacentPages, isPageReady, page.data, pageSrc])
  useReaderPrefetch({
    comicId,
    currentIndex: effectiveCurrentIndex,
    pageCount,
    pageStep,
    enabled: prefetchEnabled && manifest.isSuccess && isPageReady && pageCount > 0,
    pageQueryKey,
    requestPage
  })
  const retry = () => {
    if (manifest.isError) {
      void manifest.refetch()
      return
    }

    void page.refetch()
  }

  return {
    currentIndex: effectiveCurrentIndex,
    pageCount,
    pageSrc,
    pageWindow,
    navigationRequestId,
    isLastPage,
    isManifestLoading: manifest.isLoading,
    manifestError: manifest.isError ? manifest.error : null,
    isPageLoading: page.isLoading && !page.data,
    pageError: page.isError ? page.error : null,
    isFetching: manifest.isFetching || page.isFetching,
    goToPreviousPage,
    goToNextPage,
    goToPage,
    setObservedPage,
    pageQueryKey,
    requestPage,
    retry
  }
}
