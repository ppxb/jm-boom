import { Card, CardContent } from '@/components/ui/card'

export function ComicGridSkeleton({ count = 8 }: { count?: number }) {
  return (
    <div className="grid grid-cols-4 gap-6">
      {Array.from({ length: count }).map((_, index) => (
        <Card key={index} size="sm" className="overflow-hidden rounded-md py-0">
          <div className="aspect-square animate-pulse bg-muted" />
          <CardContent className="space-y-2 p-3">
            <div className="h-4 animate-pulse rounded bg-muted" />
            <div className="h-3 w-2/3 animate-pulse rounded bg-muted" />
          </CardContent>
        </Card>
      ))}
    </div>
  )
}
