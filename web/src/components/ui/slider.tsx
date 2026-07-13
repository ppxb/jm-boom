import * as React from 'react'
import { Slider as SliderPrimitive } from 'radix-ui'

import { cn } from '@/lib/utils'

function Slider({
  className,
  trackClassName,
  rangeClassName,
  thumbClassName,
  renderThumb,
  defaultValue,
  value,
  min = 0,
  max = 100,
  ...props
}: React.ComponentProps<typeof SliderPrimitive.Root> & {
  trackClassName?: string
  rangeClassName?: string
  thumbClassName?: string
  renderThumb?: (thumb: React.ReactElement, index: number) => React.ReactNode
}) {
  const _values = React.useMemo(
    () => (Array.isArray(value) ? value : Array.isArray(defaultValue) ? defaultValue : [min, max]),
    [value, defaultValue, min, max]
  )

  return (
    <SliderPrimitive.Root
      data-slot="slider"
      defaultValue={defaultValue}
      value={value}
      min={min}
      max={max}
      className={cn(
        'relative flex w-full touch-none items-center select-none data-disabled:opacity-50 data-vertical:h-full data-vertical:min-h-40 data-vertical:w-auto data-vertical:flex-col',
        className
      )}
      {...props}
    >
      <SliderPrimitive.Track
        data-slot="slider-track"
        className={cn(
          'relative grow overflow-hidden rounded-4xl bg-muted data-horizontal:h-3 data-horizontal:w-full data-vertical:h-full data-vertical:w-3',
          trackClassName
        )}
      >
        <SliderPrimitive.Range
          data-slot="slider-range"
          className={cn(
            'absolute bg-primary select-none data-horizontal:h-full data-vertical:w-full',
            rangeClassName
          )}
        />
      </SliderPrimitive.Track>
      {Array.from({ length: _values.length }, (_, index) => {
        const thumb = (
          <SliderPrimitive.Thumb
            data-slot="slider-thumb"
            className={cn(
              'block size-4 shrink-0 rounded-4xl border border-primary bg-white shadow-sm ring-ring/50 transition-colors select-none hover:ring-4 focus-visible:ring-4 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50',
              thumbClassName
            )}
          />
        )

        return (
          <React.Fragment key={index}>
            {renderThumb ? renderThumb(thumb, index) : thumb}
          </React.Fragment>
        )
      })}
    </SliderPrimitive.Root>
  )
}

export { Slider }
