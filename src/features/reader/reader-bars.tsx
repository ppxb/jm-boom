import { Link } from '@tanstack/react-router'
import { ArrowLeftIcon, LoaderCircleIcon, RotateCwIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import type { ReaderNextChapter } from './types'

export function ReaderTopBar({
  fallbackReadId,
  title,
  chapter,
  isFetching,
  visible,
  onBack,
  onRetry
}: {
  fallbackReadId: string
  title: string
  chapter: string
  isFetching: boolean
  visible: boolean
  onBack: () => void
  onRetry: () => void
}) {
  const displayTitle = title || `JM ${fallbackReadId}`

  return (
    <header
      className={cn(
        'absolute inset-x-0 top-0 z-30 grid h-16 grid-cols-[120px_minmax(0,1fr)_120px] items-center bg-neutral-950/85 px-4 backdrop-blur transition-all duration-200',
        visible ? 'translate-y-0 opacity-100' : 'pointer-events-none -translate-y-3 opacity-0'
      )}
      onClick={event => event.stopPropagation()}
    >
      <Button
        variant="ghost"
        size="sm"
        className="justify-self-start text-neutral-50 hover:bg-white/10"
        onClick={onBack}
      >
        <ArrowLeftIcon className="size-4" />
        返回
      </Button>

      <div className="min-w-0 text-center">
        <div className="truncate text-sm font-medium text-neutral-50">{displayTitle}</div>
        {chapter ? <div className="mt-1 truncate text-xs text-neutral-400">{chapter}</div> : null}
      </div>

      <Button
        variant="ghost"
        size="icon-sm"
        aria-label="重新加载"
        className="justify-self-end text-neutral-50 hover:bg-white/10"
        onClick={onRetry}
      >
        {isFetching ? (
          <LoaderCircleIcon className="size-4 animate-spin" />
        ) : (
          <RotateCwIcon className="size-4" />
        )}
      </Button>
    </header>
  )
}

export function ReaderBottomBar({
  title,
  nextChapter,
  albumId,
  currentIndex,
  pageCount,
  showNextChapter,
  visible
}: {
  title: string
  nextChapter: ReaderNextChapter | null
  albumId: string
  currentIndex: number
  pageCount: number
  showNextChapter: boolean
  visible: boolean
}) {
  const progress = pageCount > 0 ? ((currentIndex + 1) / pageCount) * 100 : 0

  return (
    <>
      <footer
        className={cn(
          'absolute bottom-10 left-1/2 z-30 flex w-80 -translate-x-1/2 flex-col items-center gap-2 rounded-xl border border-border/70 bg-background/85 p-3 text-center text-foreground shadow-lg backdrop-blur transition-all duration-200',
          visible ? 'translate-y-0 opacity-100' : 'pointer-events-none translate-y-3 opacity-0'
        )}
        onClick={event => event.stopPropagation()}
      >
        <div className="h-1 w-full overflow-hidden rounded-full bg-muted">
          <div
            className="h-full rounded-full bg-primary transition-[width]"
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="text-xs text-muted-foreground">
          {pageCount === 0 ? 0 : currentIndex + 1} / {pageCount}
        </div>
      </footer>

      {showNextChapter && nextChapter ? (
        <Button
          asChild
          variant="ghost"
          size="sm"
          className={cn(
            'absolute right-8 bottom-20 z-30 bg-neutral-950/85 text-neutral-50 backdrop-blur transition-all duration-200 hover:bg-white/10',
            visible ? 'translate-y-0 opacity-100' : 'pointer-events-none translate-y-3 opacity-0'
          )}
          onClick={event => event.stopPropagation()}
        >
          <Link
            to="/reader/$comicId"
            params={{ comicId: nextChapter.id }}
            replace
            search={{
              title,
              chapter: nextChapter.title,
              albumId,
              fromDetail: '1',
              pageIndex: '0',
              nextId: '',
              nextChapter: ''
            }}
          >
            下一章
          </Link>
        </Button>
      ) : null}
    </>
  )
}
