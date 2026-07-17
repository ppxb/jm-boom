import { RotateCwIcon } from 'lucide-react'
import { useEffect, useState } from 'react'

import { cn } from '@/lib/utils'
import type { ReaderPageDirection } from '@/stores/settings-store'
import { ReaderLoading } from './reader-state'
import type { ReaderWindowPage } from './types'

type ImageSize = { width: number; height: number }

// Remembers the natural dimensions of images that have already decoded so that
// remounting a page (e.g. scrolling back in the virtualized strip) can restore
// its exact aspect ratio and skip the loading placeholder instead of visibly
// re-fetching and reflowing.
const loadedImageSizes = new Map<string, ImageSize>()

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
    <div className="pointer-events-none relative h-dvh w-screen overflow-hidden">
      {pages.map(page => {
        const offset =
          pageDirection === 'rtl' ? currentIndex - page.index : page.index - currentIndex
        const isCurrent = offset === 0

        return (
          <div
            key={page.index}
            className={cn(
              'absolute inset-0 flex h-dvh w-screen items-center justify-center transition-transform duration-200 ease-out will-change-transform',
              isCurrent ? 'z-10' : 'z-0'
            )}
            style={{ transform: `translate3d(${offset * 100}%, 0, 0)` }}
          >
            <ReaderPageImage
              src={page.src}
              label={`第 ${page.index + 1} 页`}
              wrapperClassName="h-dvh w-screen"
              imageClassName="h-dvh w-screen object-contain"
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
    <div className="pointer-events-none flex h-dvh w-screen items-center justify-center overflow-hidden px-6 py-6">
      <div
        className={cn(
          'flex h-full w-full items-center justify-center gap-2',
          showNextSlot ? 'max-w-[1800px]' : 'max-w-[900px]'
        )}
      >
        <ReaderDoublePageSlot
          page={leftPage}
          isCurrent={leftIndex === currentIndex}
          label={`第 ${leftIndex + 1} 页`}
        />
        {showNextSlot ? (
          <ReaderDoublePageSlot
            page={rightPage}
            isCurrent={rightIndex === currentIndex}
            label={`第 ${rightIndex + 1} 页`}
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
        <ReaderLoading label={`正在加载${label}`} />
      )}
    </div>
  )
}

export function ReaderPageImage({
  src,
  label,
  wrapperClassName,
  placeholderClassName,
  imageClassName,
  fitContainer = false,
  loading = 'eager',
  decoding = 'async',
  showLoadingIndicator = true,
  loadingIndicatorClassName,
  onLoad
}: {
  src: string
  label: string
  wrapperClassName?: string
  placeholderClassName?: string
  imageClassName?: string
  fitContainer?: boolean
  loading?: 'eager' | 'lazy'
  decoding?: 'sync' | 'async' | 'auto'
  showLoadingIndicator?: boolean
  loadingIndicatorClassName?: string
  onLoad?: (image: HTMLImageElement) => void
}) {
  const [status, setStatus] = useState<'loading' | 'loaded' | 'error'>(() =>
    loadedImageSizes.has(src) ? 'loaded' : 'loading'
  )
  const [size, setSize] = useState<ImageSize | null>(() => loadedImageSizes.get(src) ?? null)
  const [attempt, setAttempt] = useState(0)
  const requestSrc = attempt > 0 ? appendRetryParam(src, attempt) : src

  useEffect(() => {
    const cached = loadedImageSizes.get(src)
    setStatus(cached ? 'loaded' : 'loading')
    setSize(cached ?? null)
    setAttempt(0)
  }, [src])

  return (
    <div
      className={cn(
        'relative flex items-center justify-center overflow-hidden bg-neutral-950',
        wrapperClassName,
        status !== 'loaded' && !size ? placeholderClassName : undefined
      )}
      style={size ? { aspectRatio: `${size.width} / ${size.height}` } : undefined}
    >
      {status === 'loading' && showLoadingIndicator ? (
        <ReaderLoading
          label={`正在加载${label}`}
          className={cn('absolute inset-0', loadingIndicatorClassName)}
        />
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
          'transition-opacity duration-200 select-none',
          status === 'loaded' ? 'opacity-100' : 'opacity-0',
          // Once the natural size is known, fill the aspect-ratio wrapper exactly
          // instead of letting the image compute its own height. Both paths round
          // independently, and a sub-pixel gap would expose the black backdrop as
          // a hairline between stacked pages.
          fitContainer && size ? 'absolute inset-0 h-full w-full' : imageClassName
        )}
        draggable={false}
        loading={loading}
        decoding={decoding}
        onLoad={event => {
          const image = event.currentTarget
          if (image.naturalWidth > 0 && image.naturalHeight > 0) {
            const nextSize = { width: image.naturalWidth, height: image.naturalHeight }
            loadedImageSizes.set(src, nextSize)
            setSize(nextSize)
          }
          setStatus('loaded')
          onLoad?.(image)
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
