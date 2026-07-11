import { ArrowLeftIcon, LoaderCircleIcon, RotateCwIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { ReaderChapterControls } from './reader-chapter-controls'
import { ReaderProgressSlider } from './reader-progress-slider'
import type { ReaderChapterItem } from './types'

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
        'absolute inset-x-0 top-0 z-30 grid h-20 grid-cols-[44px_minmax(0,1fr)_44px] items-center bg-neutral-950/85 px-4 backdrop-blur transition-all duration-200 sm:h-16 sm:grid-cols-[40px_minmax(0,1fr)_40px] lg:h-18 lg:grid-cols-[48px_minmax(0,1fr)_48px] lg:px-6',
        visible ? 'translate-y-0 opacity-100' : 'pointer-events-none -translate-y-3 opacity-0'
      )}
      onClick={event => event.stopPropagation()}
      onTouchMove={event => event.stopPropagation()}
    >
      <Button
        type="button"
        variant="ghost"
        size="icon-sm"
        aria-label="返回"
        className="size-11 justify-self-start rounded-md text-neutral-50 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50 sm:size-8 lg:size-11"
        onClick={onBack}
      >
        <ArrowLeftIcon className="size-5 sm:size-4 lg:size-5" />
      </Button>

      <div className="mx-auto w-full max-w-[52vw] min-w-0 text-center sm:max-w-xl lg:max-w-3xl">
        <div className="truncate text-sm font-medium text-neutral-50 lg:text-base">
          {displayTitle}
        </div>
        {chapter ? (
          <div className="mt-1 truncate text-xs text-neutral-400 lg:text-sm">{chapter}</div>
        ) : null}
      </div>

      <Button
        type="button"
        variant="ghost"
        size="icon-sm"
        aria-label="重新加载"
        className="size-11 justify-self-end rounded-md text-neutral-50 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50 sm:size-8 lg:size-11"
        onClick={onRetry}
      >
        {isFetching ? (
          <LoaderCircleIcon className="size-5 animate-spin sm:size-4 lg:size-5" />
        ) : (
          <RotateCwIcon className="size-5 sm:size-4 lg:size-5" />
        )}
      </Button>
    </header>
  )
}

export function ReaderBottomBar({
  title,
  currentReadId,
  previousChapter,
  nextChapter,
  chapters,
  albumId,
  currentIndex,
  pageCount,
  doublePageMode,
  visible,
  onPageChange
}: {
  title: string
  currentReadId: string
  previousChapter: ReaderChapterItem | null
  nextChapter: ReaderChapterItem | null
  chapters: ReaderChapterItem[]
  albumId: string
  currentIndex: number
  pageCount: number
  doublePageMode: boolean
  visible: boolean
  onPageChange: (index: number) => void
}) {
  return (
    <footer
      className={cn(
        'absolute bottom-24 left-1/2 z-30 flex w-[360px] max-w-[calc(100vw-24px)] -translate-x-1/2 flex-col rounded-2xl border border-input/20 bg-neutral-950/85 p-4 text-neutral-50 backdrop-blur transition-all duration-200 sm:bottom-8 sm:w-[480px] sm:max-w-[calc(100vw-48px)] sm:gap-2 sm:p-3 lg:w-[480px]',
        visible ? 'translate-y-0 opacity-100' : 'pointer-events-none translate-y-3 opacity-0'
      )}
      onClick={event => event.stopPropagation()}
      onTouchMove={event => event.stopPropagation()}
    >
      <ReaderProgressSlider
        currentIndex={currentIndex}
        pageCount={pageCount}
        onPageChange={onPageChange}
      />
      <ReaderChapterControls
        title={title}
        albumId={albumId}
        currentReadId={currentReadId}
        chapters={chapters}
        previousChapter={previousChapter}
        nextChapter={nextChapter}
        currentIndex={currentIndex}
        pageCount={pageCount}
        doublePageMode={doublePageMode}
      />
    </footer>
  )
}
