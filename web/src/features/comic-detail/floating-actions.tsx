import { Link } from '@tanstack/react-router'
import { BookmarkCheckIcon, BookmarkIcon, BookOpenIcon, DownloadIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import type { ComicReadingTarget } from './reading-target'

export function ComicDetailFloatingActions({
  albumId,
  readingTarget,
  isFavorite,
  downloadBusy,
  favoriteBusy,
  onFavoriteClick,
  onDownloadClick
}: {
  albumId: string
  readingTarget: ComicReadingTarget
  isFavorite: boolean
  downloadBusy: boolean
  favoriteBusy: boolean
  onFavoriteClick: () => void
  onDownloadClick: () => void
}) {
  const favoriteLabel = isFavorite ? '取消收藏' : '收藏'

  return (
    <nav
      aria-label="漫画操作"
      className="fixed bottom-4 left-1/2 z-50 inline-flex -translate-x-1/2 items-center gap-1.5 rounded-full border border-border/70 bg-background/85 p-1.5 text-foreground backdrop-blur md:hidden"
    >
      <Button asChild>
        <Link
          to="/reader/$comicId"
          params={{ comicId: readingTarget.readId }}
          search={{ albumId, page: readingTarget.page }}
        >
          <BookOpenIcon className="size-4" />
          {readingTarget.isContinue ? '继续阅读' : '开始阅读'}
        </Link>
      </Button>
      <Button
        type="button"
        variant={isFavorite ? 'secondary' : 'ghost'}
        size="icon"
        aria-label={favoriteLabel}
        title={favoriteLabel}
        disabled={favoriteBusy}
        onClick={onFavoriteClick}
      >
        {isFavorite ? (
          <BookmarkCheckIcon className="size-4" />
        ) : (
          <BookmarkIcon className="size-4" />
        )}
      </Button>
      <Button
        type="button"
        variant="ghost"
        size="icon"
        aria-label="下载"
        title="下载"
        disabled={downloadBusy}
        onClick={onDownloadClick}
      >
        <DownloadIcon className="size-4" />
      </Button>
    </nav>
  )
}
