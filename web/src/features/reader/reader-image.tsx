import { RotateCwIcon } from 'lucide-react'
import { useEffect, useLayoutEffect, useRef, useState } from 'react'

import { cn } from 'cn'
import type { ReaderPageDirection } from '@/stores/settings-store'
import { ReaderLoading } from './reader-state'
import type { ReaderWindowPage } from './types'

type ImageSize = { width: number; height: number }

const IMAGE_SIZE_CACHE_LIMIT = 1000

// Restore exact aspect ratios when virtualized pages remount, while keeping
// dimensions from older chapters from accumulating without a bound.
const loadedImageSizes = new Map<string, ImageSize>()

function rememberImageSize(src: string, size: ImageSize) {
  loadedImageSizes.delete(src)
  loadedImageSizes.set(src, size)
  if (loadedImageSizes.size > IMAGE_SIZE_CACHE_LIMIT) {
    const oldestSrc = loadedImageSizes.keys().next().value!
    loadedImageSizes.delete(oldestSrc)
  }
}

export function ReaderImageWindow({
  pages,
  currentIndex,
  pageCount,
  doublePageMode = false,
  visiblePageCount = 1,
  onPageLoaded,
  onPageReady,
  pageDirection
}: {
  pages: ReaderWindowPage[]
  currentIndex: number
  pageCount: number
  doublePageMode?: boolean
  visiblePageCount?: number
  onPageLoaded?: (page: ReaderWindowPage, image: HTMLImageElement) => void
  onPageReady?: (key: string, ready: boolean) => void
  pageDirection: ReaderPageDirection
}) {
  if (doublePageMode) {
    return (
      <ReaderDoublePageWindow
        pages={pages}
        currentIndex={currentIndex}
        pageCount={pageCount}
        visiblePageCount={visiblePageCount}
        onPageLoaded={onPageLoaded}
        onPageReady={onPageReady}
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
            key={page.key}
            className={cn(
              'absolute inset-0 flex h-dvh w-screen items-center justify-center transition-transform duration-200 ease-out will-change-transform',
              isCurrent ? 'z-10' : 'z-0'
            )}
            style={{ transform: `translate3d(${offset * 100}%, 0, 0)` }}
          >
            <ReaderPageImage
              src={page.src}
              label={`第 ${page.localIndex + 1} 页`}
              width={page.width}
              height={page.height}
              onLoad={image => onPageLoaded?.(page, image)}
              onReadyChange={ready => onPageReady?.(page.key, ready)}
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
  visiblePageCount,
  onPageLoaded,
  onPageReady,
  pageDirection
}: {
  pages: ReaderWindowPage[]
  currentIndex: number
  pageCount: number
  visiblePageCount: number
  onPageLoaded?: (page: ReaderWindowPage, image: HTMLImageElement) => void
  onPageReady?: (key: string, ready: boolean) => void
  pageDirection: ReaderPageDirection
}) {
  const pageByIndex = new Map(pages.map(page => [page.index, page]))
  const currentPage = pageByIndex.get(currentIndex) ?? null
  const nextIndex = currentIndex + 1
  const nextPage = nextIndex < pageCount ? (pageByIndex.get(nextIndex) ?? null) : null
  const showNextSlot = visiblePageCount > 1 && nextIndex < pageCount
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
          label={`第 ${(leftPage?.localIndex ?? 0) + 1} 页`}
          onPageLoaded={onPageLoaded}
          onPageReady={onPageReady}
        />
        {showNextSlot ? (
          <ReaderDoublePageSlot
            page={rightPage}
            isCurrent={rightIndex === currentIndex}
            label={`第 ${(rightPage?.localIndex ?? 0) + 1} 页`}
            onPageLoaded={onPageLoaded}
            onPageReady={onPageReady}
          />
        ) : null}
      </div>
      <ReaderHiddenImagePreloads
        pages={pages}
        visibleIndexes={visibleIndexes}
        onPageLoaded={onPageLoaded}
      />
    </div>
  )
}

function ReaderHiddenImagePreloads({
  pages,
  visibleIndexes,
  onPageLoaded
}: {
  pages: ReaderWindowPage[]
  visibleIndexes: number[]
  onPageLoaded?: (page: ReaderWindowPage, image: HTMLImageElement) => void
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
          key={page.key}
          src={page.src}
          alt=""
          className="h-px w-px"
          draggable={false}
          loading="eager"
          decoding="async"
          onLoad={event => onPageLoaded?.(page, event.currentTarget)}
        />
      ))}
    </div>
  )
}

function ReaderDoublePageSlot({
  page,
  isCurrent,
  label,
  onPageLoaded,
  onPageReady
}: {
  page: ReaderWindowPage | null
  isCurrent: boolean
  label: string
  onPageLoaded?: (page: ReaderWindowPage, image: HTMLImageElement) => void
  onPageReady?: (key: string, ready: boolean) => void
}) {
  return (
    <div className="flex h-full min-w-0 flex-1 items-center justify-center bg-neutral-950">
      {page ? (
        <ReaderPageImage
          key={page.key}
          src={page.src}
          label={label}
          width={page.width}
          height={page.height}
          onLoad={image => onPageLoaded?.(page, image)}
          onReadyChange={ready => onPageReady?.(page.key, ready)}
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
  width,
  height,
  wrapperClassName,
  placeholderClassName,
  imageClassName,
  fitContainer = false,
  loading = 'eager',
  decoding = 'async',
  showLoadingIndicator = true,
  loadingIndicatorClassName,
  onLoad,
  onReadyChange
}: {
  src: string
  label: string
  width?: number
  height?: number
  wrapperClassName?: string
  placeholderClassName?: string
  imageClassName?: string
  fitContainer?: boolean
  loading?: 'eager' | 'lazy'
  decoding?: 'sync' | 'async' | 'auto'
  showLoadingIndicator?: boolean
  loadingIndicatorClassName?: string
  onLoad?: (image: HTMLImageElement) => void
  onReadyChange?: (ready: boolean) => void
}) {
  const imageRef = useRef<HTMLImageElement | null>(null)
  const readyCallback = useRef(onReadyChange)
  useLayoutEffect(() => {
    readyCallback.current = onReadyChange
  }, [onReadyChange])
  const [ready, setReady] = useState(false)
  const [status, setStatus] = useState<'loading' | 'loaded' | 'error'>(() =>
    loadedImageSizes.has(src) ? 'loaded' : 'loading'
  )
  const [loadedSize, setSize] = useState<ImageSize | null>(() => loadedImageSizes.get(src) ?? null)
  const size = loadedSize ?? (width && height && width > 0 && height > 0 ? { width, height } : null)
  const [attempt, setAttempt] = useState(0)
  const requestSrc = attempt > 0 ? appendRetryParam(src, attempt) : src

  useEffect(() => {
    const cached = loadedImageSizes.get(src)
    if (cached) rememberImageSize(src, cached)
    setStatus(cached ? 'loaded' : 'loading')
    setSize(cached ?? null)
    setAttempt(0)
    setReady(Boolean(imageRef.current?.complete && imageRef.current.naturalWidth > 0))
    return () => readyCallback.current?.(false)
  }, [src])

  useEffect(() => {
    onReadyChange?.(ready)
  }, [onReadyChange, ready])

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
              setReady(false)
              setAttempt(value => value + 1)
            }}
          >
            <RotateCwIcon className="size-3.5" />
            重试
          </button>
        </div>
      ) : null}

      <img
        ref={imageRef}
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
            rememberImageSize(src, nextSize)
            setSize(nextSize)
          }
          setStatus('loaded')
          setReady(true)
          onLoad?.(image)
        }}
        onError={() => {
          setStatus('error')
          setReady(false)
        }}
      />
    </div>
  )
}

function appendRetryParam(src: string, attempt: number) {
  const separator = src.includes('?') ? '&' : '?'
  return `${src}${separator}retry=${attempt}`
}
