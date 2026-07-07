import { Link } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'

import { ComicCover } from '@/components/comic-cover'
import { Card, CardContent } from '@/components/ui/card'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
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
