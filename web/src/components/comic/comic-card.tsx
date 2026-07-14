import { Link } from '@tanstack/react-router'

import { OverflowTooltip } from '@/components/overflow-tooltip'
import { Card, CardContent } from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Progress } from '@/components/ui/progress'
import type { ComicCardSummary } from '@/domain/comic'
import { cn } from '@/lib/utils'
import { ComicCover } from './comic-cover'

type ComicCardProps = {
  comic: ComicCardSummary
  ratio?: 'portrait' | 'square'
  showIdBadge?: boolean
  metadata?: React.ReactNode
  progress?: number
  coverOverlay?: React.ReactNode
  selectable?: boolean
  selected?: boolean
  onSelect?: (id: string, checked: boolean) => void
  onOpen?: () => void
  linkProps?: {
    to: string
    params: Record<string, string>
    search?: Record<string, string | number>
  }
  className?: string
}

export function ComicCard({
  comic,
  ratio = 'portrait',
  showIdBadge = false,
  metadata,
  progress,
  coverOverlay,
  selectable = false,
  selected = false,
  onSelect,
  onOpen,
  linkProps,
  className
}: ComicCardProps) {
  const card = (
    <Card
      size="sm"
      aria-pressed={selectable ? selected : undefined}
      role={selectable ? 'button' : onOpen ? 'link' : undefined}
      tabIndex={selectable || onOpen ? 0 : undefined}
      className={cn(
        'group gap-0 overflow-hidden py-0 transition-shadow hover:cursor-pointer hover:shadow-xl',
        className
      )}
      onClick={selectable ? () => onSelect?.(comic.id, !selected) : onOpen}
      onKeyDown={
        selectable || onOpen
          ? event => {
              if (event.key !== 'Enter' && event.key !== ' ') return
              event.preventDefault()
              if (selectable) {
                onSelect?.(comic.id, !selected)
              } else {
                onOpen?.()
              }
            }
          : undefined
      }
    >
      <div className="relative">
        {selectable ? (
          <div className="absolute top-4 right-4 z-30">
            <Checkbox
              checked={selected}
              className="data-checked:border-green-500 data-checked:bg-green-500 dark:data-checked:border-green-500 dark:data-checked:bg-green-500"
              onClick={event => event.stopPropagation()}
              onKeyDown={event => event.stopPropagation()}
              onCheckedChange={checked => onSelect?.(comic.id, checked === true)}
            />
          </div>
        ) : null}

        <ComicCover
          id={comic.id}
          title={comic.title}
          image={comic.image}
          ratio={ratio}
          showIdBadge={showIdBadge}
          className={ratio === 'square' ? 'w-full rounded-none' : undefined}
        />

        {coverOverlay}

        {progress !== undefined ? (
          <div className="absolute right-2 bottom-2 left-2 z-30">
            <Progress value={Math.min(100, progress * 100)} className="h-1.5 bg-black/40" />
          </div>
        ) : null}
      </div>

      <CardContent className="space-y-1.5 p-3">
        <OverflowTooltip asChild content={comic.title}>
          <div className="truncate text-sm font-semibold">{comic.title}</div>
        </OverflowTooltip>
        {metadata}
      </CardContent>
    </Card>
  )

  if (selectable) {
    return <div className="block">{card}</div>
  }

  if (linkProps) {
    return (
      <Link
        to={linkProps.to}
        params={linkProps.params}
        search={linkProps.search}
        className="block focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
      >
        {card}
      </Link>
    )
  }

  return card
}
