import type { ComicSummary } from '@/domain/comic'
import { ComicCard } from './comic-card'

export function ComicGrid({ items }: { items: ComicSummary[] }) {
  return (
    <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 sm:gap-4 lg:grid-cols-4 lg:gap-6">
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
          metadata={
            <p className="line-clamp-1 text-xs text-muted-foreground">{item.author || 'N/A'}</p>
          }
        />
      ))}
    </div>
  )
}
