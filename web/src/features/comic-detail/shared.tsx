import { ComicRail, ComicRailItem } from '@/components/comic'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { UI } from '@/lib/constants'

export { ComicCover } from '@/components/comic'

export function ComicDetailSkeleton() {
  return (
    <div className="space-y-10">
      <section className="grid gap-6 md:grid-cols-[220px_minmax(0,1fr)] lg:grid-cols-[240px_minmax(0,1fr)] lg:gap-8">
        <div className="relative mx-auto aspect-3/4 w-full max-w-60 md:max-w-none">
          <Skeleton className="size-full" />
          <Skeleton className="absolute top-2 left-2 h-6 w-20 rounded-full" />
        </div>
        <div className="min-w-0 space-y-5 py-1 text-center md:text-left">
          <div className="space-y-2">
            <Skeleton className="mx-auto h-8 w-3/4 sm:h-9 lg:h-10 md:mx-0" />
            <div className="flex justify-center md:justify-start">
              <Skeleton className="h-4 w-36" />
            </div>
          </div>
          <div className="h-px bg-border" />
          <StatsSkeleton />
          <div className="h-px bg-border" />
          <div className="space-y-2">
            <Skeleton className="mx-auto h-4 w-full max-w-3xl md:mx-0" />
            <Skeleton className="mx-auto h-4 w-5/6 max-w-2xl md:mx-0" />
            <Skeleton className="mx-auto h-4 w-2/3 max-w-xl md:mx-0" />
          </div>
          <div className="hidden gap-2 md:flex">
            <Skeleton className="h-9 w-28" />
            <Skeleton className="h-9 w-24" />
            <Skeleton className="h-9 w-20" />
          </div>
          <div className="space-y-3">
            <PillGroupSkeleton prominent />
            <PillGroupSkeleton />
            <PillGroupSkeleton prominent />
          </div>
        </div>
      </section>
      <ChapterSkeletonList />
      <RelatedSkeletonRail />
    </div>
  )
}

function StatsSkeleton() {
  return (
    <div className="grid grid-cols-2 rounded-md bg-card/60 sm:grid-cols-4">
      {Array.from({ length: 4 }).map((_, index) => (
        <div key={index} className="flex flex-col items-center justify-center space-y-1 p-4">
          <Skeleton className="h-3 w-14" />
          <Skeleton className="h-7 w-12" />
        </div>
      ))}
    </div>
  )
}

function PillGroupSkeleton({ prominent = false }: { prominent?: boolean }) {
  const badgeHeight = prominent ? 'h-7 md:h-5' : 'h-5'

  return (
    <div className="flex flex-col items-start gap-2 md:flex-row md:items-center">
      <Skeleton className="h-4 w-8 md:w-10" />
      <div className="flex flex-wrap gap-2">
        <Skeleton className={`${badgeHeight} w-16`} />
        <Skeleton className={`${badgeHeight} w-20`} />
        <Skeleton className={`${badgeHeight} w-14`} />
      </div>
    </div>
  )
}

export function CommentSkeletonList() {
  return (
    <div className="min-h-full space-y-3 pb-2">
      {Array.from({ length: UI.COMMENT_SKELETON_COUNT }).map((_, index) => (
        <div key={index} className="space-y-3 px-px py-1">
          <div className="space-y-2">
            <Skeleton className="h-4 w-40" />
            <Skeleton className="h-3 w-24" />
          </div>
          <div className="space-y-2">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-2/3" />
          </div>
        </div>
      ))}
    </div>
  )
}

function ChapterSkeletonList() {
  return (
    <section className="space-y-4">
      <div className="flex items-center gap-2">
        <Skeleton className="h-7 w-16" />
        <Skeleton className="h-7 w-28" />
      </div>
      <div className="space-y-2">
        {Array.from({ length: UI.CHAPTER_PAGE_SIZE }).map((_, index) => (
          <Card key={index} size="sm" className="py-0">
            <CardContent className="flex items-center justify-between gap-4 p-4">
              <div className="min-w-0 flex-1 space-y-2">
                <Skeleton className="h-4 w-1/2 max-w-64" />
                <Skeleton className="h-3 w-20" />
              </div>
              <Skeleton className="h-6 w-20 rounded-full" />
            </CardContent>
          </Card>
        ))}
      </div>
    </section>
  )
}

function RelatedSkeletonRail() {
  return (
    <section className="min-w-0 space-y-6">
      <div className="space-y-1">
        <Skeleton className="h-7 w-24" />
        <Skeleton className="h-4 w-20" />
      </div>
      <ComicRail>
        {Array.from({ length: 4 }).map((_, index) => (
          <ComicRailItem key={index}>
            <Card size="sm" className="gap-0 overflow-hidden py-0">
              <div className="relative">
                <Skeleton className="aspect-square w-full rounded-none" />
                <Skeleton className="absolute top-2 left-2 h-6 w-20 rounded-full" />
              </div>
              <CardContent className="space-y-1.5 p-3">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-3 w-2/3" />
              </CardContent>
            </Card>
          </ComicRailItem>
        ))}
      </ComicRail>
    </section>
  )
}
