import { LoaderCircleIcon, RotateCwIcon } from 'lucide-react'
import { useEffect, useState } from 'react'

import { cn } from '@/lib/utils'
import type { ReaderPageDirection } from '@/stores/settings-store'
import type { ReaderWindowPage } from './types'

export function ReaderImageWindow({
  pages,
  currentIndex,
  pageCount,
  doublePageMode = false,
  pageDirection
}: {
  pages: ReaderWindowPage[]
  currentIndex: number
  pageCount: number
  doublePageMode?: boolean
  pageDirection: ReaderPageDirection
}) {
  if (doublePageMode) {
    return (
      <ReaderDoublePageWindow
        pages={pages}
        currentIndex={currentIndex}
        pageCount={pageCount}
        pageDirection={pageDirection}
      />
    )
  }

  return (
    <div className="pointer-events-none relative h-screen w-screen overflow-hidden">
      {pages.map(page => {
        const offset =
          pageDirection === 'rtl' ? currentIndex - page.index : page.index - currentIndex
        const isCurrent = offset === 0

        return (
          <div
            key={page.index}
            className={cn(
              'absolute inset-0 flex h-screen w-screen items-center justify-center transition-transform duration-200 ease-out will-change-transform',
              isCurrent ? 'z-10' : 'z-0'
            )}
            style={{ transform: `translate3d(${offset * 100}%, 0, 0)` }}
          >
            <ReaderPageImage
              src={page.src}
              label={`第 ${page.index + 1} 张`}
              wrapperClassName="h-screen w-screen"
              imageClassName="h-screen w-screen object-contain"
              loading="eager"
              decoding={isCurrent ? 'sync' : 'async'}
            />
          </div>
        )
      })}
    </div>
  )
}

function ReaderDoublePageWindow({
  pages,
  currentIndex,
  pageCount,
  pageDirection
}: {
  pages: ReaderWindowPage[]
  currentIndex: number
  pageCount: number
  pageDirection: ReaderPageDirection
}) {
  const pageByIndex = new Map(pages.map(page => [page.index, page]))
  const currentPage = pageByIndex.get(currentIndex) ?? null
  const nextIndex = currentIndex + 1
  const nextPage = nextIndex < pageCount ? (pageByIndex.get(nextIndex) ?? null) : null
  const showNextSlot = nextIndex < pageCount
  const leftPage = pageDirection === 'rtl' && showNextSlot ? nextPage : currentPage
  const rightPage = pageDirection === 'rtl' ? currentPage : nextPage
  const leftIndex = pageDirection === 'rtl' && showNextSlot ? nextIndex : currentIndex
  const rightIndex = pageDirection === 'rtl' ? currentIndex : nextIndex
  const visibleIndexes = showNextSlot ? [leftIndex, rightIndex] : [currentIndex]

  return (
    <div className="pointer-events-none flex h-screen w-screen items-center justify-center overflow-hidden px-6 py-6">
      <div
        className={cn(
          'flex h-full w-full items-center justify-center gap-2',
          showNextSlot ? 'max-w-[1800px]' : 'max-w-[900px]'
        )}
      >
        <ReaderDoublePageSlot
          page={leftPage}
          isCurrent={leftIndex === currentIndex}
          label={`第 ${leftIndex + 1} 张`}
        />
        {showNextSlot ? (
          <ReaderDoublePageSlot
            page={rightPage}
            isCurrent={rightIndex === currentIndex}
            label={`第 ${rightIndex + 1} 张`}
          />
        ) : null}
      </div>
      <ReaderHiddenImagePreloads pages={pages} visibleIndexes={visibleIndexes} />
    </div>
  )
}

function ReaderHiddenImagePreloads({
  pages,
  visibleIndexes
}: {
  pages: ReaderWindowPage[]
  visibleIndexes: number[]
}) {
  const visibleIndexSet = new Set(visibleIndexes)
  const preloadPages = pages.filter(page => !visibleIndexSet.has(page.index))

  if (preloadPages.length === 0) {
    return null
  }

  return (
    <div className="pointer-events-none absolute h-px w-px overflow-hidden opacity-0">
      {preloadPages.map(page => (
        <img
          key={page.index}
          src={page.src}
          alt=""
          className="h-px w-px"
          draggable={false}
          loading="eager"
          decoding="async"
        />
      ))}
    </div>
  )
}

function ReaderDoublePageSlot({
  page,
  isCurrent,
  label
}: {
  page: ReaderWindowPage | null
  isCurrent: boolean
  label: string
}) {
  return (
    <div className="flex h-full min-w-0 flex-1 items-center justify-center bg-neutral-950">
      {page ? (
        <ReaderPageImage
          src={page.src}
          label={label}
          wrapperClassName="h-full w-full"
          imageClassName="max-h-full max-w-full object-contain"
          loading="eager"
          decoding={isCurrent ? 'sync' : 'async'}
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center text-xs text-neutral-500">
          正在准备{label}
        </div>
      )}
    </div>
  )
}

export function ReaderPageImage({
  src,
  label,
  wrapperClassName,
  imageClassName,
  loading = 'eager',
  decoding = 'async',
  onLoad
}: {
  src: string
  label: string
  wrapperClassName?: string
  imageClassName?: string
  loading?: 'eager' | 'lazy'
  decoding?: 'sync' | 'async' | 'auto'
  onLoad?: () => void
}) {
  const [status, setStatus] = useState<'loading' | 'loaded' | 'error'>('loading')
  const [attempt, setAttempt] = useState(0)
  const requestSrc = attempt > 0 ? appendRetryParam(src, attempt) : src

  useEffect(() => {
    setStatus('loading')
    setAttempt(0)
  }, [src])

  return (
    <div
      className={cn(
        'relative flex items-center justify-center overflow-hidden bg-neutral-950',
        wrapperClassName
      )}
    >
      {status === 'loading' ? (
        <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 text-neutral-400">
          <LoaderCircleIcon className="size-6 animate-spin" />
          <span className="text-xs">正在加载{label}</span>
        </div>
      ) : null}

      {status === 'error' ? (
        <div className="pointer-events-auto absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 text-neutral-400">
          <span className="text-xs">{label}加载失败</span>
          <button
            type="button"
            className="inline-flex h-8 items-center gap-1.5 rounded-md border border-white/15 bg-white/5 px-3 text-xs text-neutral-200 hover:bg-white/10"
            onClick={event => {
              event.stopPropagation()
              setStatus('loading')
              setAttempt(value => value + 1)
            }}
          >
            <RotateCwIcon className="size-3.5" />
            重试
          </button>
        </div>
      ) : null}

      <img
        src={requestSrc}
        alt=""
        className={cn(
          'select-none transition-opacity duration-200',
          status === 'loaded' ? 'opacity-100' : 'opacity-0',
          imageClassName
        )}
        draggable={false}
        loading={loading}
        decoding={decoding}
        onLoad={() => {
          setStatus('loaded')
          onLoad?.()
        }}
        onError={() => setStatus('error')}
      />
    </div>
  )
}

function appendRetryParam(src: string, attempt: number) {
  const separator = src.includes('?') ? '&' : '?'
  return `${src}${separator}retry=${attempt}`
}
