import { useEffect, useMemo } from 'react'

import type { ComicReadManifestPage } from '@/lib/api/reader'
import { READER } from '@/lib/constants'
import { clearReaderPreloadScope, setReaderPreloadScope } from '@/lib/reader-preload'

export function useReaderPreload({
  readId,
  pages,
  currentIndex
}: {
  readId: string
  pages: ComicReadManifestPage[]
  currentIndex: number
}) {
  const scope = `reader:${readId}`
  const paths = useMemo(
    () => selectPreloadPaths(pages, currentIndex),
    [currentIndex, pages]
  )

  useEffect(() => {
    setReaderPreloadScope(scope, paths)
  }, [paths, scope])

  useEffect(() => () => clearReaderPreloadScope(scope), [scope])
}

function selectPreloadPaths(pages: ComicReadManifestPage[], currentIndex: number) {
  const paths: string[] = []

  for (let offset = 1; offset <= READER.PREFETCH_AHEAD_PAGES; offset += 1) {
    const path = pages[currentIndex + offset]?.path

    if (path) {
      paths.push(path)
    }
  }

  for (let offset = 1; offset <= READER.PREFETCH_BEHIND_PAGES; offset += 1) {
    const path = pages[currentIndex - offset]?.path

    if (path) {
      paths.push(path)
    }
  }

  return paths
}
