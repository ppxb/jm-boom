import { Link } from '@tanstack/react-router'
import { ChevronLeftIcon, ChevronRightIcon, ListIcon } from 'lucide-react'
import { useState, type ReactNode } from 'react'

import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { ReaderChapterDrawer } from './reader-chapter-drawer'
import { toReaderChapterSearch } from './reader-chapter-link'
import { ReaderSettingsMenu } from './reader-settings-menu'
import type { ReaderChapterItem } from './types'

const CHAPTER_BUTTON_CLASS =
  'h-11 w-11 rounded-md px-0 text-xs text-neutral-200 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50 disabled:text-neutral-500 sm:h-8 sm:w-auto sm:px-2 sm:text-xs'

export function ReaderChapterControls({
  title,
  albumId,
  currentReadId,
  chapters,
  previousChapter,
  nextChapter,
  currentIndex,
  pageCount,
  doublePageMode
}: {
  title: string
  albumId: string
  currentReadId: string
  chapters: ReaderChapterItem[]
  previousChapter: ReaderChapterItem | null
  nextChapter: ReaderChapterItem | null
  currentIndex: number
  pageCount: number
  doublePageMode: boolean
}) {
  const [chapterDrawerOpen, setChapterDrawerOpen] = useState(false)
  const hasChapterList = chapters.length > 1
  const hasChapterNavigation = hasChapterList || previousChapter != null || nextChapter != null
  const pageLabel =
    doublePageMode && currentIndex + 1 < pageCount
      ? `${currentIndex + 1}-${currentIndex + 2} / ${pageCount}`
      : `${currentIndex + 1} / ${pageCount}`

  return (
    <>
      <div className="flex w-full items-center justify-between gap-2 sm:gap-3">
        <div className="flex min-w-0 items-center gap-1.5">
          <ChapterNavButton
            label="上一章"
            albumId={albumId}
            chapter={hasChapterNavigation ? previousChapter : null}
          >
            <ChevronLeftIcon className="size-5 sm:size-4" />
            <span className="hidden sm:inline">上一章</span>
          </ChapterNavButton>

          <ChapterNavButton
            label="下一章"
            albumId={albumId}
            chapter={hasChapterNavigation ? nextChapter : null}
          >
            <span className="hidden sm:inline">下一章</span>
            <ChevronRightIcon className="size-5 sm:size-4" />
          </ChapterNavButton>

          <Button
            type="button"
            variant="ghost"
            size="xs"
            aria-label="章节目录"
            disabled={!hasChapterList}
            className={CHAPTER_BUTTON_CLASS}
            onClick={() => setChapterDrawerOpen(true)}
          >
            <ListIcon className="size-5 sm:size-4" />
            <span className="hidden sm:inline">章节目录</span>
          </Button>

          <ReaderSettingsMenu />
        </div>

        <div className="shrink-0 text-sm text-neutral-300 tabular-nums sm:min-w-20 sm:text-right sm:text-xs">
          {pageLabel}
        </div>
      </div>

      <ReaderChapterDrawer
        open={chapterDrawerOpen}
        onOpenChange={setChapterDrawerOpen}
        title={title}
        albumId={albumId}
        currentReadId={currentReadId}
        chapters={chapters}
      />
    </>
  )
}

function ChapterNavButton({
  label,
  albumId,
  chapter,
  children
}: {
  label: string
  albumId: string
  chapter: ReaderChapterItem | null
  children: ReactNode
}) {
  if (!chapter) {
    return (
      <Button
        variant="ghost"
        size="xs"
        aria-label={label}
        disabled
        className={CHAPTER_BUTTON_CLASS}
      >
        {children}
      </Button>
    )
  }

  return (
    <Button asChild variant="ghost" size="xs" className={cn(CHAPTER_BUTTON_CLASS)}>
      <Link
        to="/reader/$comicId"
        params={{ comicId: chapter.id }}
        aria-label={label}
        replace
        search={toReaderChapterSearch({ albumId })}
      >
        {children}
      </Link>
    </Button>
  )
}
