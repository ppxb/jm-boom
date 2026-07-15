import { createFileRoute } from '@tanstack/react-router'

import { ReaderPage } from '@/features/reader/reader-page'
import { SourceReaderPage } from '@/features/source-reader/page'
import type { ReaderSearch } from '@/features/reader/types'

export const Route = createFileRoute('/reader/$comicId')({
  validateSearch: (search: Record<string, unknown>): ReaderSearch => ({
    albumId: typeof search.albumId === 'string' ? search.albumId : '',
    page: parseOptionalPage(search.page),
    sourceId: parseOptionalString(search.sourceId),
    mangaKey: parseOptionalString(search.mangaKey)
  }),
  component: ReaderRoute
})

function ReaderRoute() {
  const { comicId } = Route.useParams()
  const search = Route.useSearch()

  if (search.sourceId && search.mangaKey) {
    return (
      <SourceReaderPage
        sourceId={search.sourceId}
        mangaKey={search.mangaKey}
        chapterKey={comicId}
        initialPage={search.page}
      />
    )
  }

  return <ReaderPage comicId={comicId} search={search} />
}

function parseOptionalString(value: unknown) {
  if (typeof value !== 'string') return undefined
  const text = value.trim()
  return text.length > 0 ? text : undefined
}

function parseOptionalPage(value: unknown) {
  const page = Number(value)

  return Number.isSafeInteger(page) && page > 0 ? page : undefined
}
