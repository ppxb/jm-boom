import { Card, CardContent } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'

export function ComicGridSkeleton({ count = 8 }: { count?: number }) {
  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4 lg:grid-cols-4 lg:gap-6">
      {Array.from({ length: count }).map((_, index) => (
        <Card key={index} size="sm" className="gap-0 overflow-hidden py-0">
          <Skeleton className="aspect-square w-full rounded-none" />
          <CardContent className="space-y-1.5 p-3">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-3 w-2/3" />
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
