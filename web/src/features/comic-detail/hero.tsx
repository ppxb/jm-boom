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
import type { ComicDetail } from '@/domain/comic'
import { getComicDisplayChapterCount } from '@/lib/comic'
import { formatNumber } from '@/lib/format'
import { ComicDetailFloatingActions } from './floating-actions'
import type { ComicReadingTarget } from './reading-target'
import { ComicCover } from './shared'

export function ComicHero({
  comic,
  readingTarget,
  isFavorite,
  onCommentsClick,
  onDownloadClick,
  onFavoriteClick,
  onCoverSettled,
  downloadBusy = false
}: {
  comic: ComicDetail
  readingTarget: ComicReadingTarget
  isFavorite: boolean
  onCommentsClick: () => void
  onDownloadClick: () => void
  onFavoriteClick: () => void
  onCoverSettled: () => void
  downloadBusy?: boolean
}) {
  return (
    <section className="grid gap-6 md:grid-cols-[220px_minmax(0,1fr)] lg:grid-cols-[240px_minmax(0,1fr)] lg:gap-8">
      <ComicCover
        id={comic.id}
        title={comic.title}
        image={comic.image}
        loading="eager"
        onImageSettled={onCoverSettled}
        className="mx-auto w-full max-w-60 md:max-w-none"
      />

      <div className="min-w-0 space-y-5 py-1 text-center md:text-left">
        <Badge variant="default">JM {comic.id}</Badge>

        <div className="space-y-2">
          <h1 className="text-2xl leading-tight font-bold tracking-normal sm:text-3xl lg:text-4xl">
            {comic.title}
          </h1>
          <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground md:justify-start">
            <UserRoundIcon className="size-4" />
            <SearchLinks items={comic.authors} fallback="N/A" className="min-w-0" />
          </div>
        </div>

        <Separator />
        <StatsRow comic={comic} onCommentsClick={onCommentsClick} />
        <Separator />

        <p className="mx-auto max-w-3xl text-left text-sm leading-7 text-muted-foreground md:mx-0">
          {comic.description || '暂无简介'}
        </p>

        <div className="md:hidden">
          <ComicDetailFloatingActions
            albumId={comic.id}
            readingTarget={readingTarget}
            isFavorite={isFavorite}
            downloadBusy={downloadBusy}
            onFavoriteClick={onFavoriteClick}
            onDownloadClick={onDownloadClick}
          />
        </div>

        <div className="hidden flex-wrap gap-2 md:flex">
          <Button asChild>
            <Link
              to="/reader/$comicId"
              params={{ comicId: readingTarget.readId }}
              search={{
                albumId: comic.id,
                page: readingTarget.page
              }}
            >
              <BookOpenIcon className="size-4" />
              {readingTarget.isContinue ? '继续阅读' : '开始阅读'}
            </Link>
          </Button>
          <Button variant={isFavorite ? 'secondary' : 'outline'} onClick={onFavoriteClick}>
            {isFavorite ? (
              <BookmarkCheckIcon className="size-4" />
            ) : (
              <BookmarkIcon className="size-4" />
            )}
            {isFavorite ? '已收藏' : '收藏'}
          </Button>
          <Button variant="outline" disabled={downloadBusy} onClick={onDownloadClick}>
            <DownloadIcon className="size-4" />
            下载
          </Button>
        </div>

        <div className="space-y-3">
          <PillGroup title="标签" items={comic.tags} mobileProminent />
          <PillGroup title="角色" items={comic.actors} variant="secondary" />
          <PillGroup title="作品" items={comic.works} variant="secondary" mobileProminent />
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
      value: formatNumber(comic.commentCount),
      icon: MessageCircleIcon,
      onClick: onCommentsClick
    },
    {
      id: 'chapters',
      label: '章节',
      value: formatNumber(getComicDisplayChapterCount(comic.chapters)),
      icon: LayersIcon
    }
  ]

  return (
    <div className="grid grid-cols-2 rounded-md bg-card/60 text-center text-sm sm:grid-cols-4">
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
          <div key={stat.id} className="flex min-w-0 items-stretch">
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
            {index < stats.length - 1 ? (
              <Separator orientation="vertical" className="hidden sm:block" />
            ) : null}
          </div>
        )
      })}
    </div>
  )
}

function PillGroup({
  title,
  items,
  variant = 'outline',
  mobileProminent = false
}: {
  title: string
  items: string[]
  variant?: 'outline' | 'secondary'
  mobileProminent?: boolean
}) {
  if (items.length === 0) {
    return null
  }

  return (
    <div className="flex flex-col items-start gap-2 md:flex-row md:items-center">
      <span
        className={
          mobileProminent
            ? 'text-sm text-muted-foreground md:w-10 md:text-xs'
            : 'text-xs text-muted-foreground md:w-10'
        }
      >
        {title}
      </span>
      <div className="flex flex-wrap justify-start gap-2">
        {items.map(item => (
          <Badge
            key={`${title}-${item}`}
            variant={variant}
            asChild
            className={mobileProminent ? 'h-7 px-3 text-sm md:h-5 md:px-2 md:text-xs' : undefined}
          >
            <Link
              to="/explore/search"
              search={{
                q: item
              }}
            >
              {item}
            </Link>
          </Badge>
        ))}
      </div>
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
            to="/explore/search"
            search={{
              q: item
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
