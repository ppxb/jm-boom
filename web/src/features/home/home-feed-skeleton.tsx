import { ComicRail, ComicRailItem } from '@/components/comic'
import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

export function HomeFeedSkeleton() {
  return (
    <>
      {Array.from({ length: 2 }).map((_, index) => (
        <section key={index} className="space-y-6">
          <Skeleton className="h-7 w-32" />
          <ComicRail>
            {Array.from({ length: 4 }).map((_, itemIndex) => (
              <ComicRailItem key={itemIndex}>
                <Card size="sm" className="gap-0 overflow-hidden py-0">
                  <Skeleton className="aspect-square w-full rounded-none" />
                  <CardContent className="space-y-1.5 p-3">
                    <Skeleton className="h-4 w-full" />
                    <Skeleton className="h-3 w-2/3" />
                  </CardContent>
                </Card>
              </ComicRailItem>
            ))}
          </ComicRail>
        </section>
      ))}
    </>
  )
}
