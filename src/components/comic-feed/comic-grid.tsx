import { Link } from '@tanstack/react-router'

import { ComicCover } from '@/components/comic-cover'
import { OverflowTooltip } from '@/components/overflow-tooltip'
import { Card, CardContent } from '@/components/ui/card'
import type { FeedComic } from '@/lib/api/home'

export function ComicGrid({ items }: { items: FeedComic[] }) {
  return (
    <div className="grid grid-cols-4 gap-6">
      {items.map(item => (
        <ComicCard key={item.id} item={item} />
      ))}
    </div>
  )
}

function ComicCard({ item }: { item: FeedComic }) {
  return (
    <Link
      to="/comic/$comicId"
      params={{ comicId: item.id }}
      className="block focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
    >
      <Card
        size="sm"
        className="gap-0 overflow-hidden py-0 transition-shadow hover:cursor-pointer hover:shadow-xl"
      >
        <ComicCover
          id={item.id}
          title={item.title}
          image={item.image}
          className="w-full rounded-none"
          ratio="square"
          showIdBadge
        />
        <CardContent className="space-y-1.5 p-3">
          <OverflowTooltip asChild content={item.title}>
            <div className="truncate text-sm font-semibold">{item.title}</div>
          </OverflowTooltip>
          <p className="line-clamp-1 text-xs text-muted-foreground">{item.author || 'N/A'}</p>
        </CardContent>
      </Card>
    </Link>
  )
}
