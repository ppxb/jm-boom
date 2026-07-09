import { Link } from '@tanstack/react-router'

import { ComicCover } from '@/components/comic-cover'
import { OverflowTooltip } from '@/components/overflow-tooltip'
import { Card, CardContent } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { cn, formatDate } from '@/lib/utils'
import { type ReadingHistoryItem } from '@/stores/reading-history-store'

interface HistoryCardProps {
  item: ReadingHistoryItem
  isSelecting: boolean
  isSelected: boolean
  onSelectionChange: (comicId: string, checked: boolean) => void
}

export function HistoryCard({
  item,
  isSelecting,
  isSelected,
  onSelectionChange
}: HistoryCardProps) {
  const coverSrc = item.coverUrl?.trim() ?? ''
  const progress = ((item.pageIndex + 1) / item.pageCount) * 100
  const title = item.title
  const card = (
    <Card
      size="sm"
      aria-pressed={isSelecting ? isSelected : undefined}
      role={isSelecting ? 'button' : undefined}
      tabIndex={isSelecting ? 0 : undefined}
      className={cn(
        'gap-0 overflow-hidden py-0 transition-shadow hover:cursor-pointer hover:shadow-xl'
      )}
      onClick={isSelecting ? () => onSelectionChange(item.comicId, !isSelected) : undefined}
      onKeyDown={
        isSelecting
          ? event => {
              if (event.key !== 'Enter' && event.key !== ' ') {
                return
              }

              event.preventDefault()
              onSelectionChange(item.comicId, !isSelected)
            }
          : undefined
      }
    >
      <div className="relative">
        {isSelecting ? (
          <div className="absolute top-4 right-4 z-30">
            <Checkbox
              checked={isSelected}
              className="data-checked:border-green-500 data-checked:bg-green-500 dark:data-checked:border-green-500 dark:data-checked:bg-green-500"
              onClick={event => event.stopPropagation()}
              onKeyDown={event => event.stopPropagation()}
              onCheckedChange={checked => onSelectionChange(item.comicId, checked === true)}
            />
          </div>
        ) : null}
        <ComicCover id={item.comicId} title={title} image={coverSrc} ratio="square" showIdBadge />
        <div className="absolute right-2 bottom-2 left-2 z-20">
          <div className="h-1 overflow-hidden rounded-full bg-black/40">
            <div className="h-full rounded-full bg-primary" style={{ width: `${progress}%` }} />
          </div>
        </div>
      </div>
      <CardContent className="space-y-1.5 p-3">
        <OverflowTooltip asChild content={title}>
          <div className="truncate text-sm font-semibold">{title}</div>
        </OverflowTooltip>
        <p className="line-clamp-1 text-xs text-muted-foreground">{item.chapterTitle}</p>
        <p className="text-xs text-muted-foreground">
          {item.pageIndex + 1}/{item.pageCount} • {formatDate(item.updatedAt)}
        </p>
      </CardContent>
    </Card>
  )

  if (isSelecting) {
    return <div className="block">{card}</div>
  }

  return (
    <Link
      to="/reader/$comicId"
      params={{ comicId: item.chapterId }}
      search={{
        title,
        chapter: item.chapterTitle,
        albumId: item.albumId,
        fromDetail: '',
        pageIndex: String(item.pageIndex),
        nextId: '',
        nextChapter: ''
      }}
      className="block"
    >
      {card}
    </Link>
  )
}
