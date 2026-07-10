import { createFileRoute } from '@tanstack/react-router'

import { ComicDetailPage } from '@/features/comic-detail/page'

export const Route = createFileRoute('/_app/comic/$comicId')({
  component: ComicDetailRoute
})

function ComicDetailRoute() {
  const { comicId } = Route.useParams()

  return <ComicDetailPage comicId={comicId} />
}
