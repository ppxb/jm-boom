import type { ReaderBoundary } from './types'

export function ReaderChapterBoundary({
  boundary,
  previous = false,
  onRetry
}: {
  boundary: ReaderBoundary
  previous?: boolean
  onRetry: () => void
}) {
  return (
    <div
      className="flex h-32 flex-col items-center justify-center gap-2 text-sm text-neutral-400"
      onClick={event => event.stopPropagation()}
    >
      <span>
        {boundary.chapter
          ? (boundary.loading ? '正在加载：' : '') + boundary.chapter.title
          : previous
            ? '已到第一章'
            : '已到最后一章'}
      </span>
      {boundary.error ? (
        <span role="alert" className="text-xs">
          {boundary.error}
        </span>
      ) : null}
      {boundary.chapter && !boundary.loading ? (
        <button
          type="button"
          className="rounded border border-white/20 px-3 py-1.5 text-neutral-200"
          onClick={onRetry}
        >
          {boundary.error ? '重试' : previous ? '加载上一章' : '加载下一章'}
        </button>
      ) : null}
    </div>
  )
}
