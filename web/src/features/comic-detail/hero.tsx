import { Link } from '@tanstack/react-router'
import {
  BookOpenIcon,
  BookmarkIcon,
  BookmarkCheckIcon,
  DownloadIcon,
  EyeIcon,
  HeartIcon,
  LayersIcon,
  MessageCircleIcon,
  UserRoundIcon,
  type LucideIcon
} from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import type { ComicDetail } from '@/lib/api/comic'
import {
  getComicDisplayChapterCount,
  resolveComicAlbumId,
  resolveComicStartReadingId
} from '@/lib/comic'
import { formatNumber } from '@/lib/format'
import { ComicCover } from './shared'

export function ComicHero({
  comic,
  onCommentsClick,
  onDownloadClick,
  onFavoriteClick,
  downloadBusy = false,
  favoriteBusy = false
}: {
  comic: ComicDetail
  onCommentsClick: () => void
  onDownloadClick: () => void
  onFavoriteClick: () => void
  downloadBusy?: boolean
  favoriteBusy?: boolean
}) {
  const albumId = resolveComicAlbumId(comic)
  const startReadingId = resolveComicStartReadingId(comic)

  return (
    <section className="grid grid-cols-[240px_minmax(0,1fr)] gap-8">
      <ComicCover id={comic.id} title={comic.title} image={comic.image} className="w-full" />

      <div className="min-w-0 space-y-5 py-1">
        <Badge variant="default">JM {comic.id}</Badge>

        <div className="space-y-2">
          <h1 className="text-4xl leading-tight font-bold tracking-normal">{comic.title}</h1>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <UserRoundIcon className="size-4" />
            <SearchLinks items={comic.author} fallback="N/A" className="min-w-0" />
          </div>
        </div>

        <Separator />
        <StatsRow comic={comic} onCommentsClick={onCommentsClick} />
        <Separator />

        <p className="max-w-3xl text-sm leading-7 text-muted-foreground">
          {comic.description || '暂无简介'}
        </p>

        <div className="flex flex-wrap gap-2">
          <Button asChild>
            <Link
              to="/reader/$comicId"
              params={{ comicId: startReadingId }}
              search={{
                albumId,
                pageIndex: '0'
              }}
            >
              <BookOpenIcon className="size-4" />
              开始阅读
            </Link>
          </Button>
          <Button
            variant={comic.isFavorite ? 'secondary' : 'outline'}
            onClick={onFavoriteClick}
            disabled={favoriteBusy}
          >
            {comic.isFavorite ? (
              <BookmarkCheckIcon className="size-4" />
            ) : (
              <BookmarkIcon className="size-4" />
            )}
            {comic.isFavorite ? '已收藏' : '收藏'}
          </Button>
          <Button variant="outline" disabled={downloadBusy} onClick={onDownloadClick}>
            <DownloadIcon className="size-4" />
            下载
          </Button>
        </div>

        <div className="space-y-3">
          <PillGroup title="标签" items={comic.tags} />
          <PillGroup title="角色" items={comic.actors} variant="secondary" />
          <PillGroup title="作品" items={comic.works} variant="secondary" />
        </div>
      </div>
    </section>
  )
}

function StatsRow({ comic, onCommentsClick }: { comic: ComicDetail; onCommentsClick: () => void }) {
  const stats: Array<{
    id: string
    label: string
    value: string
    icon: LucideIcon
    onClick?: () => void
  }> = [
    { id: 'views', label: '浏览', value: formatNumber(comic.totalViews), icon: EyeIcon },
    { id: 'likes', label: '喜欢', value: formatNumber(comic.likes), icon: HeartIcon },
    {
      id: 'comments',
      label: '评论',
      value: formatNumber(comic.commentTotal),
      icon: MessageCircleIcon,
      onClick: onCommentsClick
    },
    {
      id: 'chapters',
      label: '章节',
      value: formatNumber(getComicDisplayChapterCount(comic.series)),
      icon: LayersIcon
    }
  ]

  return (
    <div className="flex items-stretch rounded-md bg-card/60 text-center text-sm">
      {stats.map((stat, index) => {
        const content = (
          <>
            <div className="flex items-center justify-center gap-2 text-xs font-medium text-muted-foreground">
              <stat.icon className="size-4" />
              {stat.label}
            </div>
            <div className="text-xl font-semibold">{stat.value}</div>
          </>
        )

        return (
          <div key={stat.id} className="flex min-w-0 flex-1 items-stretch">
            {stat.onClick ? (
              <button
                type="button"
                className="flex min-w-0 flex-1 cursor-pointer flex-col items-center justify-center space-y-1 rounded-sm p-4 transition-colors hover:bg-muted focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
                onClick={stat.onClick}
              >
                {content}
              </button>
            ) : (
              <div className="flex min-w-0 flex-1 flex-col items-center justify-center space-y-1 p-4">
                {content}
              </div>
            )}
            {index < stats.length - 1 ? <Separator orientation="vertical" /> : null}
          </div>
        )
      })}
    </div>
  )
}

function PillGroup({
  title,
  items,
  variant = 'outline'
}: {
  title: string
  items: string[]
  variant?: 'outline' | 'secondary'
}) {
  if (items.length === 0) {
    return null
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="w-10 text-xs text-muted-foreground">{title}</span>
      {items.map(item => (
        <Badge key={`${title}-${item}`} variant={variant} asChild>
          <Link
            to="/search"
            search={{
              keyword: item,
              page: 1,
              sortBy: 1
            }}
          >
            {item}
          </Link>
        </Badge>
      ))}
    </div>
  )
}

function SearchLinks({
  items,
  fallback,
  className
}: {
  items: string[]
  fallback: string
  className?: string
}) {
  if (items.length === 0) {
    return <span className={className}>{fallback}</span>
  }

  return (
    <span className={`flex min-w-0 flex-wrap items-center gap-x-1 gap-y-1 ${className ?? ''}`}>
      {items.map((item, index) => (
        <span key={item} className="inline-flex min-w-0 items-center gap-x-1">
          {index > 0 ? <span className="text-muted-foreground/70">/</span> : null}
          <Link
            to="/search"
            search={{
              keyword: item,
              page: 1,
              sortBy: 1
            }}
            className="max-w-md truncate underline-offset-4 hover:text-foreground hover:underline focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
          >
            {item}
          </Link>
        </span>
      ))}
    </span>
  )
}
