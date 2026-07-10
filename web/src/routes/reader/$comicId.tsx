import { createFileRoute } from '@tanstack/react-router'

import { ReaderPage } from '@/features/reader/reader-page'
import type { ReaderSearch } from '@/features/reader/types'

export const Route = createFileRoute('/reader/$comicId')({
  validateSearch: (search: Record<string, unknown>): ReaderSearch => ({
    title: typeof search.title === 'string' ? search.title : '',
    chapter: typeof search.chapter === 'string' ? search.chapter : '',
    albumId: typeof search.albumId === 'string' ? search.albumId : '',
    fromDetail: typeof search.fromDetail === 'string' ? search.fromDetail : '',
    pageIndex: typeof search.pageIndex === 'string' ? search.pageIndex : '',
    nextId: typeof search.nextId === 'string' ? search.nextId : '',
    nextChapter: typeof search.nextChapter === 'string' ? search.nextChapter : ''
  }),
  component: ReaderRoute
})

function ReaderRoute() {
  const { comicId } = Route.useParams()
  const search = Route.useSearch()

  return <ReaderPage comicId={comicId} search={search} />
}
