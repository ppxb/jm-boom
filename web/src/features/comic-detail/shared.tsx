import { Skeleton } from '@/components/ui/skeleton'
import { UI } from '@/lib/constants'

export { ComicCover } from '@/components/comic'

export function SectionHeading({ title, description }: { title: string; description: string }) {
  return (
    <div className="flex items-end justify-between gap-4">
      <div className="space-y-1">
        <h2 className="text-xl font-semibold tracking-normal">{title}</h2>
        <p className="text-sm text-muted-foreground">{description}</p>
      </div>
    </div>
  )
}

export function ComicDetailSkeleton() {
  return (
    <div className="space-y-10">
      <section className="grid grid-cols-[240px_minmax(0,1fr)] gap-8">
        <Skeleton className="aspect-3/4" />
        <div className="space-y-5 py-1">
          <Skeleton className="h-5 w-56" />
          <div className="space-y-3">
            <Skeleton className="h-10 w-2/3" />
            <Skeleton className="h-4 w-64" />
          </div>
          <div className="h-px bg-border" />
          <Skeleton className="h-24 max-w-3xl" />
          <div className="h-px bg-border" />
          <div className="space-y-2">
            <Skeleton className="h-4 max-w-3xl" />
            <Skeleton className="h-4 max-w-2xl" />
            <Skeleton className="h-4 max-w-xl" />
          </div>
        </div>
      </section>
      <div className="grid grid-cols-[minmax(0,1fr)_320px] gap-8">
        <div className="space-y-8">
          <ChapterSkeletonList />
        </div>
        <Skeleton className="h-80" />
      </div>
    </div>
  )
}

export function CommentSkeletonList() {
  return (
    <div className="space-y-3">
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
      <div className="space-y-2">
        <Skeleton className="h-6 w-24" />
        <Skeleton className="h-4 w-32" />
      </div>
      <div className="space-y-2">
        {Array.from({ length: 4 }).map((_, index) => (
          <Skeleton key={index} className="h-18" />
        ))}
      </div>
    </section>
  )
}
