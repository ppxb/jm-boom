import { createFileRoute } from '@tanstack/react-router'

import { ComicDetailPage } from '@/features/comic-detail/page'
import { SourceComicDetailPage } from '@/features/source-comic-detail/page'

type ComicDetailSearch = {
  sourceId?: string
}

export const Route = createFileRoute('/_app/comic/$comicId')({
  validateSearch: validateSearchParams,
  component: ComicDetailRoute
})

function ComicDetailRoute() {
  const { comicId } = Route.useParams()
  const { sourceId } = Route.useSearch()

  if (sourceId) {
    return <SourceComicDetailPage sourceId={sourceId} mangaKey={comicId} />
  }

  return <ComicDetailPage comicId={comicId} />
}

function validateSearchParams(search: Record<string, unknown>): ComicDetailSearch {
  if (typeof search.sourceId !== 'string') return {}
  const sourceId = search.sourceId.trim()
  return sourceId.length > 0 ? { sourceId } : {}
}
