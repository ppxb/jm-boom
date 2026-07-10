import type { FeedComic } from '@/lib/api/home'
import { ComicCard } from './comic-card'

export function ComicGrid({ items }: { items: FeedComic[] }) {
  return (
    <div className="grid grid-cols-4 gap-6">
      {items.map(item => (
        <ComicCard
          key={item.id}
          comic={item}
          ratio="square"
          showIdBadge
          linkProps={{
            to: '/comic/$comicId',
            params: { comicId: item.id }
          }}
          metadata={<p className="line-clamp-1 text-xs text-muted-foreground">{item.author || 'N/A'}</p>}
        />
      ))}
    </div>
  )
}
