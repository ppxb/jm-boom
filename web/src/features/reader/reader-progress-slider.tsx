import { useEffect, useState } from 'react'

import { Progress } from '@/components/ui/progress'
import { cn } from '@/lib/utils'

export function ReaderProgressSlider({
  currentIndex,
  pageCount,
  onPageChange
}: {
  currentIndex: number
  pageCount: number
  onPageChange: (index: number) => void
}) {
  const rangeMax = Math.max(pageCount - 1, 0)
  const progress = rangeMax > 0 ? (currentIndex / rangeMax) * 100 : 0
  const [isHovered, setIsHovered] = useState(false)
  const [isScrubbing, setIsScrubbing] = useState(false)
  const [isFocused, setIsFocused] = useState(false)
  const [previewIndex, setPreviewIndex] = useState(currentIndex)
  const safePreviewIndex = Math.min(Math.max(previewIndex, 0), rangeMax)
  const previewPercent = rangeMax > 0 ? (safePreviewIndex / rangeMax) * 100 : 0
  const showKnob = pageCount > 1 && (isHovered || isScrubbing || isFocused)
  const showTooltip = pageCount > 1 && (isScrubbing || isFocused)

  useEffect(() => {
    setPreviewIndex(currentIndex)
  }, [currentIndex])

  return (
    <div
      className="relative flex h-5 w-full items-center"
      onPointerEnter={() => setIsHovered(true)}
      onPointerLeave={() => setIsHovered(false)}
    >
      <div
        className={cn(
          'pointer-events-none absolute -top-9 z-10 min-w-14 rounded-md bg-neutral-50 px-2 py-1 text-xs font-medium text-neutral-950 shadow-lg transition-opacity duration-150 after:absolute after:top-full after:left-1/2 after:-translate-x-1/2 after:border-4 after:border-transparent after:border-t-neutral-50',
          'whitespace-nowrap tabular-nums',
          showTooltip ? 'opacity-100' : 'opacity-0'
        )}
        style={{
          left: `${previewPercent}%`,
          transform: 'translateX(-50%)'
        }}
      >
        第{safePreviewIndex + 1}张
      </div>
      <Progress
        value={progress}
        className="h-0.5 bg-white/20 [&_[data-slot=progress-indicator]]:bg-neutral-50"
      />
      <input
        type="range"
        aria-label="阅读进度"
        aria-valuetext={`第 ${currentIndex + 1} 张，共 ${pageCount} 张`}
        min={0}
        max={rangeMax}
        step={1}
        value={currentIndex}
        disabled={pageCount <= 1}
        className={cn(
          'absolute inset-x-0 top-1/2 h-5 -translate-y-1/2 cursor-pointer appearance-none bg-transparent disabled:cursor-default disabled:opacity-60',
          '[&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-track]:bg-transparent',
          '[&::-webkit-slider-runnable-track]:h-0.5 [&::-webkit-slider-runnable-track]:bg-transparent',
          '[&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full',
          isScrubbing
            ? '[&::-moz-range-thumb]:size-2.5 [&::-webkit-slider-thumb]:mt-[-4px] [&::-webkit-slider-thumb]:size-2.5'
            : '[&::-moz-range-thumb]:size-1.5 [&::-webkit-slider-thumb]:mt-[-2px] [&::-webkit-slider-thumb]:size-1.5',
          showKnob
            ? '[&::-moz-range-thumb]:bg-neutral-50 [&::-webkit-slider-thumb]:bg-neutral-50 [&::-webkit-slider-thumb]:shadow'
            : '[&::-moz-range-thumb]:bg-transparent [&::-webkit-slider-thumb]:bg-transparent [&::-webkit-slider-thumb]:shadow-none'
        )}
        onPointerDown={event => {
          event.currentTarget.setPointerCapture(event.pointerId)
          setIsScrubbing(true)
        }}
        onPointerUp={event => {
          event.currentTarget.releasePointerCapture(event.pointerId)
          setIsScrubbing(false)
        }}
        onPointerCancel={() => setIsScrubbing(false)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => {
          setIsFocused(false)
          setIsScrubbing(false)
        }}
        onChange={event => {
          const nextIndex = Number(event.currentTarget.value)
          setPreviewIndex(nextIndex)
          onPageChange(nextIndex)
        }}
      />
    </div>
  )
}
