import { ComicGridSkeleton } from '@/components/comic'
import { Skeleton } from '@/components/ui/skeleton'

export function HomeFeedSkeleton() {
  return (
    <>
      {Array.from({ length: 2 }).map((_, index) => (
        <section key={index} className="space-y-6">
          <Skeleton className="h-7 w-32" />
          <ComicGridSkeleton />
        </section>
      ))}
    </>
  )
}
