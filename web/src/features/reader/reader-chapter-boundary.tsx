import { InfoIcon, LoaderCircleIcon } from 'lucide-react'

import type { ReaderBoundary, ReaderChapterItem } from './types'

export function ReaderChapterBoundary({
  currentChapter,
  boundary,
  previous = false,
  onRetry
}: {
  currentChapter: ReaderChapterItem
  boundary: ReaderBoundary
  previous?: boolean
  onRetry?: () => void
}) {
  const topChapter = previous ? boundary.chapter : currentChapter
  const bottomChapter = previous ? currentChapter : boundary.chapter

  return (
    <section
      aria-label="章节过渡"
      data-reader-transition
      className="mx-auto my-auto flex min-h-60 w-full max-w-[460px] flex-col items-center justify-center gap-6 px-8 py-10 text-center"
    >
      {topChapter ? (
        <ChapterLabel label={previous ? '上一章' : '已读完'} chapter={topChapter} />
      ) : (
        <NoChapter label="没有上一章了" />
      )}
      {bottomChapter ? (
        <ChapterLabel label={previous ? '当前章节' : '下一章'} chapter={bottomChapter} />
      ) : (
        <NoChapter label="已读完全部章节" />
      )}
      {boundary.loading ? (
        <div role="status" className="flex items-center gap-2 text-sm text-neutral-400">
          <LoaderCircleIcon aria-hidden className="size-4 animate-spin" />
          正在加载章节…
        </div>
      ) : boundary.error ? (
        <p role="alert" className="text-sm break-words text-neutral-400">
          {boundary.error}
        </p>
      ) : null}
      {boundary.chapter && boundary.error && !boundary.loading && onRetry ? (
        <button
          type="button"
          className="relative z-30 rounded-full bg-white/10 px-5 py-2.5 text-sm text-neutral-100 hover:bg-white/15 focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-white/60"
          onClick={event => {
            event.stopPropagation()
            onRetry()
          }}
        >
          重试
        </button>
      ) : null}
    </section>
  )
}

function ChapterLabel({ label, chapter }: { label: string; chapter: ReaderChapterItem }) {
  return (
    <div className="w-full space-y-1.5">
      <p className="text-sm text-neutral-400">{label}</p>
      <p className="line-clamp-5 text-xl leading-relaxed font-medium break-words text-neutral-100">
        {chapter.title || '正文'}
      </p>
    </div>
  )
}

function NoChapter({ label }: { label: string }) {
  return (
    <div className="inline-flex items-center gap-3 rounded-lg border border-white/15 px-4 py-3 text-sm text-neutral-300">
      <InfoIcon aria-hidden className="size-4 shrink-0" />
      <span>{label}</span>
    </div>
  )
}
