import { createFileRoute } from '@tanstack/react-router'

import { ReaderPage } from '@/features/reader/reader-page'
import type { ReaderSearch } from '@/features/reader/types'

export const Route = createFileRoute('/reader/$comicId')({
  validateSearch: (search: Record<string, unknown>): ReaderSearch => ({
    albumId: typeof search.albumId === 'string' ? search.albumId : '',
    pageIndex: typeof search.pageIndex === 'string' ? search.pageIndex : ''
  }),
  component: ReaderRoute
})

function ReaderRoute() {
  const { comicId } = Route.useParams()
  const search = Route.useSearch()

  return <ReaderPage comicId={comicId} search={search} />
}
