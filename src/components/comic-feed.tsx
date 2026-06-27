import { ImageIcon, RefreshCwIcon } from 'lucide-react'
import { Link } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'

import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import type { FeedComic } from '@/lib/api/home'
import { useSettingsStore } from '@/stores/settings-store'

export function FeedHeader({
  title,
  description,
  isFetching,
  onRefresh
}: {
  title: string
  description: string
  isFetching?: boolean
  onRefresh?: () => void
}) {
  return (
    <div className="mb-4 flex items-start justify-between gap-4">
      <div className="min-w-0 space-y-2">
        <h1 className="text-3xl font-bold tracking-normal">{title}</h1>
        <p className="text-sm leading-6 text-muted-foreground">{description}</p>
      </div>
      {onRefresh ? (
        <Button
          type="button"
          variant="outline"
          size="icon"
          disabled={isFetching}
          onClick={onRefresh}
          aria-label="刷新"
          className="cursor-pointer"
        >
          <RefreshCwIcon className={isFetching ? 'animate-spin' : undefined} />
        </Button>
      ) : null}
    </div>
  )
}

export function ComicGrid({ items }: { items: FeedComic[] }) {
  const hideCovers = useSettingsStore(state => state.hideCovers)

  return (
    <div className="grid grid-cols-4 gap-6">
      {items.map(item => (
        <ComicCard key={item.id} item={item} hideCover={hideCovers} />
      ))}
    </div>
  )
}

function ComicCard({ item, hideCover }: { item: FeedComic; hideCover: boolean }) {
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
        <ComicCover id={item.id} title={item.title} image={item.image} hideCover={hideCover} />
        <CardContent className="space-y-1.5 p-3">
          <OverflowTooltipTitle title={item.title} />
          <p className="line-clamp-1 text-xs text-muted-foreground">{item.author || 'N/A'}</p>
        </CardContent>
      </Card>
    </Link>
  )
}

function OverflowTooltipTitle({ title }: { title: string }) {
  const titleRef = useRef<HTMLDivElement>(null)
  const [isOverflowing, setIsOverflowing] = useState(false)
  const titleElement = (
    <div ref={titleRef} className="truncate text-sm font-semibold">
      {title}
    </div>
  )

  useEffect(() => {
    const element = titleRef.current

    if (!element) {
      return
    }

    const target = element
    let frame = 0

    function updateOverflow() {
      cancelAnimationFrame(frame)
      frame = requestAnimationFrame(() => {
        setIsOverflowing(target.scrollWidth > target.clientWidth + 1)
      })
    }

    updateOverflow()

    const resizeObserver = new ResizeObserver(updateOverflow)
    resizeObserver.observe(target)

    return () => {
      cancelAnimationFrame(frame)
      resizeObserver.disconnect()
    }
  }, [isOverflowing, title])

  if (!isOverflowing) {
    return titleElement
  }

  return (
    <Tooltip>
      <TooltipTrigger asChild>{titleElement}</TooltipTrigger>
      <TooltipContent side="top">{title}</TooltipContent>
    </Tooltip>
  )
}

function ComicCover({
  id,
  title,
  image,
  hideCover
}: {
  id: string
  title: string
  image: string
  hideCover: boolean
}) {
  const [hasImageError, setHasImageError] = useState(false)
  const shouldShowImage = image.length > 0 && !hasImageError

  useEffect(() => {
    setHasImageError(false)
  }, [image])

  return (
    <div className="relative aspect-square overflow-hidden bg-muted">
      {shouldShowImage ? (
        <img
          src={image}
          alt={title}
          loading="lazy"
          decoding="async"
          referrerPolicy="no-referrer"
          className="h-full w-full object-cover"
          onError={() => setHasImageError(true)}
        />
      ) : (
        <CoverPlaceholder />
      )}
      {hideCover ? <CoverMask /> : null}
      <div className="absolute top-2 left-2 z-20 rounded-full border border-input/80 bg-background/45 px-2 py-1 text-[10px] backdrop-blur">
        JM {id}
      </div>
    </div>
  )
}

function CoverPlaceholder() {
  return (
    <div className="flex h-full items-center justify-center bg-muted text-muted-foreground">
      <ImageIcon className="size-6" />
    </div>
  )
}

function CoverMask() {
  return (
    <div className="absolute inset-0 z-10 flex items-center justify-center bg-muted/90 text-muted-foreground backdrop-blur-sm">
      <ImageIcon className="size-6" />
    </div>
  )
}

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

export function StatePanel({
  title,
  description,
  onRetry
}: {
  title: string
  description?: string
  onRetry?: () => void
}) {
  return (
    <div className="flex min-h-40 flex-col items-center justify-center gap-3 rounded-md border border-dashed border-border/70 bg-card/60 px-6 py-8 text-center">
      <p className="text-sm font-medium">{title}</p>
      {description ? <p className="max-w-md text-sm text-muted-foreground">{description}</p> : null}
      {onRetry ? (
        <Button type="button" variant="outline" size="sm" onClick={onRetry}>
          重试
        </Button>
      ) : null}
    </div>
  )
}
