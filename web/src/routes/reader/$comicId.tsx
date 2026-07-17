import { createFileRoute } from '@tanstack/react-router'

import { ReaderPage } from '@/features/reader/reader-page'
import type { ReaderSearch } from '@/features/reader/types'

export const Route = createFileRoute('/reader/$comicId')({
  validateSearch: (search: Record<string, unknown>): ReaderSearch => ({
    albumId: typeof search.albumId === 'string' ? search.albumId : '',
    page: parseOptionalPage(search.page)
  }),
  component: ReaderRoute
})

function ReaderRoute() {
  const { comicId } = Route.useParams()
  const search = Route.useSearch()

  return <ReaderPage key={`${comicId}:${search.page ?? 1}`} comicId={comicId} search={search} />
}

function parseOptionalPage(value: unknown) {
  const page = Number(value)

  return Number.isSafeInteger(page) && page > 0 ? page : undefined
}
