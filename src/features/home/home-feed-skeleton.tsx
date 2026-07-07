import { ComicGridSkeleton } from '@/components/comic-feed'

export function HomeFeedSkeleton() {
  return (
    <>
      {Array.from({ length: 2 }).map((_, index) => (
        <section key={index} className="space-y-6">
          <div className="space-y-2">
            <div className="h-6 w-40 animate-pulse rounded bg-muted" />
            <div className="h-4 w-24 animate-pulse rounded bg-muted" />
          </div>
          <ComicGridSkeleton />
        </section>
      ))}
    </>
  )
}
