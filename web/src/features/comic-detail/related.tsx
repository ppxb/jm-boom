import { Link } from '@tanstack/react-router'

import { ComicCard, ComicRail, ComicRailItem } from '@/components/comic'
import { Badge } from '@/components/ui/badge'
import type { RelatedComic } from '@/domain/comic'

export function RelatedPanel({ items }: { items: RelatedComic[] }) {
  return (
    <section className="min-w-0 space-y-6">
      <div className="space-y-1">
        <h2 className="text-xl font-semibold tracking-normal">相关推荐</h2>
        <p className="text-sm text-muted-foreground">{items.length} 部作品</p>
      </div>
      {items.length === 0 ? (
        <p className="text-sm text-muted-foreground">暂无相关推荐</p>
      ) : (
        <ComicRail>
          {items.map(item => (
            <ComicRailItem key={item.id}>
              <ComicCard
                comic={item}
                ratio="square"
                showIdBadge
                linkProps={{
                  to: '/comic/$comicId',
                  params: { comicId: item.id }
                }}
                metadata={
                  <p className="line-clamp-1 text-xs text-muted-foreground">
                    {item.authors.join(' / ') || 'N/A'}
                  </p>
                }
              />
              <div className="mt-2 flex flex-wrap gap-1.5">
                {Array.from(
                  new Set([
                    ...item.authors,
                    ...item.categories,
                    ...item.tags,
                    ...item.works,
                    ...item.actors
                  ])
                )
                  .slice(0, 6)
                  .map(value => (
                    <Badge
                      key={value}
                      variant="secondary"
                      className="max-w-full"
                      render={
                        <Link to="/explore/search" search={{ q: value }} title={value}>
                          <span className="truncate">{value}</span>
                        </Link>
                      }
                    />
                  ))}
              </div>
            </ComicRailItem>
          ))}
        </ComicRail>
      )}
    </section>
  )
}
