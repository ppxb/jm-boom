import { Link } from '@tanstack/react-router'

import { OverflowTooltip } from '@/components/overflow-tooltip'
import { Badge } from '@/components/ui/badge'
import type { RelatedComic } from '@/lib/api/comic'
import { ComicCover } from './shared'

export function RelatedPanel({ items }: { items: RelatedComic[] }) {
  return (
    <section className="space-y-4">
      <div className="space-y-1 px-1">
        <h2 className="text-xl font-semibold tracking-normal">相关推荐</h2>
        <p className="text-sm text-muted-foreground">{items.length} 部作品</p>
      </div>
      <div className="space-y-3">
        {items.length === 0 ? (
          <p className="px-1 text-sm text-muted-foreground">暂无相关推荐</p>
        ) : (
          items.map(item => <RelatedItem key={item.id} item={item} />)
        )}
      </div>
    </section>
  )
}

function RelatedItem({ item }: { item: RelatedComic }) {
  return (
    <Link
      to="/comic/$comicId"
      params={{ comicId: item.id }}
      className="grid grid-cols-[64px_minmax(0,1fr)] gap-3 rounded-md p-1"
    >
      <ComicCover id={item.id} title={item.title} image={item.image} className="w-16" />
      <div className="min-w-0 space-y-1 self-center">
        <OverflowTooltip asChild content={item.title}>
          <div className="truncate text-sm font-semibold">{item.title}</div>
        </OverflowTooltip>
        <div className="truncate text-xs text-muted-foreground">{item.author || 'N/A'}</div>
        <Badge variant="outline">JM {item.id}</Badge>
      </div>
    </Link>
  )
}
