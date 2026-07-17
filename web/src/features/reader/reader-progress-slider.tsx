import { useEffect, useRef, useState } from 'react'

import { Slider } from '@/components/ui/slider'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'

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
  const [isScrubbing, setIsScrubbing] = useState(false)
  const [isFocused, setIsFocused] = useState(false)
  const [previewIndex, setPreviewIndex] = useState(currentIndex)
  const previewIndexRef = useRef(currentIndex)
  const isInteracting = isScrubbing || isFocused
  const safePreviewIndex = clampIndex(previewIndex, rangeMax)
  const displayIndex = isInteracting ? safePreviewIndex : currentIndex
  const showTooltip = pageCount > 1 && isScrubbing

  useEffect(() => {
    if (isInteracting) {
      return
    }

    previewIndexRef.current = currentIndex
    setPreviewIndex(currentIndex)
  }, [currentIndex, isInteracting])

  const updatePreview = (index: number) => {
    const nextIndex = clampIndex(index, rangeMax)
    previewIndexRef.current = nextIndex
    setPreviewIndex(nextIndex)
  }

  const commitPreview = (index: number) => {
    const nextIndex = clampIndex(index, rangeMax)
    previewIndexRef.current = nextIndex
    setPreviewIndex(nextIndex)

    if (nextIndex !== currentIndex) {
      onPageChange(nextIndex)
    }
  }

  const cancelPreview = () => {
    previewIndexRef.current = currentIndex
    setPreviewIndex(currentIndex)
  }

  return (
    <Tooltip open={showTooltip}>
      <div className="flex h-5 w-full items-center">
        <Slider
          aria-label="阅读进度"
          aria-valuetext={`第 ${displayIndex + 1} 张，共 ${pageCount} 张`}
          min={0}
          max={rangeMax}
          step={1}
          value={[displayIndex]}
          disabled={pageCount <= 1}
          className="h-full cursor-pointer"
          trackClassName="bg-black/40 data-horizontal:h-1"
          rangeClassName="bg-primary"
          thumbClassName="size-2 border-0 bg-primary shadow-none ring-0 hover:ring-0 focus-visible:ring-0"
          renderThumb={thumb => <TooltipTrigger asChild>{thumb}</TooltipTrigger>}
          onValueChange={values => updatePreview(values[0] ?? currentIndex)}
          onValueCommit={values => commitPreview(values[0] ?? currentIndex)}
          onPointerDown={() => setIsScrubbing(true)}
          onPointerUp={() => setIsScrubbing(false)}
          onPointerCancel={() => {
            cancelPreview()
            setIsScrubbing(false)
          }}
          onFocus={() => setIsFocused(true)}
          onBlur={() => {
            setIsFocused(false)
            setIsScrubbing(false)
          }}
        />
      </div>
      <TooltipContent side="top" sideOffset={6}>
        第 {safePreviewIndex + 1} 页
      </TooltipContent>
    </Tooltip>
  )
}

function clampIndex(index: number, rangeMax: number) {
  return Math.min(Math.max(index, 0), rangeMax)
}
