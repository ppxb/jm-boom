import { Settings2Icon } from 'lucide-react'
import type { ReactNode } from 'react'

import { Button } from '@/components/ui/button'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '@/components/ui/dropdown-menu'
import { Switch } from '@/components/ui/switch'
import { cn } from '@/lib/utils'
import {
  READER_AUTO_READ_PAGE_INTERVAL_RANGE,
  READER_AUTO_READ_STRIP_DISTANCE_RANGE,
  READER_AUTO_READ_STRIP_INTERVAL_RANGE,
  useSettingsStore
} from '@/stores/settings-store'

const READER_SETTING_BUTTON_CLASS =
  'h-11 w-11 rounded-md px-0 text-xs text-neutral-200 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50 sm:h-7 sm:w-auto sm:px-2'

const READER_SETTING_ITEM_CLASS =
  'text-neutral-100 focus:bg-white/10 focus:text-neutral-50 [&_svg]:text-neutral-300'

const READER_SETTING_SWITCH_CLASS =
  'border-white/20 data-checked:bg-neutral-100 data-unchecked:bg-white/20 dark:data-unchecked:bg-white/20 [&_[data-slot=switch-thumb]]:data-checked:bg-neutral-950 [&_[data-slot=switch-thumb]]:data-unchecked:bg-neutral-100'

export function ReaderSettingsMenu() {
  const readerReadMode = useSettingsStore(state => state.readerReadMode)
  const readerPageDirection = useSettingsStore(state => state.readerPageDirection)
  const readerDoublePageMode = useSettingsStore(state => state.readerDoublePageMode)
  const readerAutoReadEnabled = useSettingsStore(state => state.readerAutoReadEnabled)
  const readerAutoReadStripIntervalMs = useSettingsStore(
    state => state.readerAutoReadStripIntervalMs
  )
  const readerAutoReadPageIntervalMs = useSettingsStore(state => state.readerAutoReadPageIntervalMs)
  const readerAutoReadStripDistancePercent = useSettingsStore(
    state => state.readerAutoReadStripDistancePercent
  )
  const setReaderReadMode = useSettingsStore(state => state.setReaderReadMode)
  const setReaderPageDirection = useSettingsStore(state => state.setReaderPageDirection)
  const setReaderDoublePageMode = useSettingsStore(state => state.setReaderDoublePageMode)
  const setReaderAutoReadEnabled = useSettingsStore(state => state.setReaderAutoReadEnabled)
  const setReaderAutoReadStripIntervalMs = useSettingsStore(
    state => state.setReaderAutoReadStripIntervalMs
  )
  const setReaderAutoReadPageIntervalMs = useSettingsStore(
    state => state.setReaderAutoReadPageIntervalMs
  )
  const setReaderAutoReadStripDistancePercent = useSettingsStore(
    state => state.setReaderAutoReadStripDistancePercent
  )
  const isSingleMode = readerReadMode === 'single'

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          type="button"
          variant="ghost"
          size="xs"
          aria-label="阅读设置"
          className={READER_SETTING_BUTTON_CLASS}
        >
          <Settings2Icon className="size-5 sm:size-3.5" />
          <span className="hidden sm:inline">阅读设置</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        side="top"
        align="start"
        className="w-56 border border-white/10 bg-neutral-950/95 text-neutral-50 shadow-2xl backdrop-blur-xl"
      >
        <DropdownMenuLabel className="text-neutral-400">阅读模式</DropdownMenuLabel>
        <DropdownMenuRadioGroup value={readerReadMode} onValueChange={setReaderReadMode}>
          <DropdownMenuRadioItem value="single" className={READER_SETTING_ITEM_CLASS}>
            单页
          </DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="strip" className={READER_SETTING_ITEM_CLASS}>
            竖向阅读
          </DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
        {isSingleMode ? (
          <>
            <DropdownMenuSeparator className="bg-white/10" />
            <DropdownMenuLabel className="text-neutral-400">翻页方向</DropdownMenuLabel>
            <div className="grid grid-cols-2 gap-1 px-1 pb-1">
              <ReaderDirectionButton
                selected={readerPageDirection === 'ltr'}
                onClick={() => setReaderPageDirection('ltr')}
              >
                从左向右
              </ReaderDirectionButton>
              <ReaderDirectionButton
                selected={readerPageDirection === 'rtl'}
                onClick={() => setReaderPageDirection('rtl')}
              >
                从右向左
              </ReaderDirectionButton>
            </div>
          </>
        ) : null}
        <DropdownMenuSeparator className="bg-white/10" />
        <div className="flex items-center justify-between gap-3 px-3 py-2">
          <div className="min-w-0">
            <div className="text-sm text-neutral-100">双页阅读</div>
            <div className="mt-0.5 text-xs text-neutral-500">仅在单页模式中生效</div>
          </div>
          <Switch
            checked={readerDoublePageMode}
            disabled={!isSingleMode}
            className={READER_SETTING_SWITCH_CLASS}
            onCheckedChange={setReaderDoublePageMode}
          />
        </div>
        <DropdownMenuSeparator className="bg-white/10" />
        <div className="flex items-center justify-between gap-3 px-3 py-2">
          <div className="min-w-0">
            <div className="text-sm text-neutral-100">自动阅读</div>
            <div className="mt-0.5 text-xs text-neutral-500">隐藏控制栏时自动推进</div>
          </div>
          <Switch
            checked={readerAutoReadEnabled}
            className={READER_SETTING_SWITCH_CLASS}
            onCheckedChange={setReaderAutoReadEnabled}
          />
        </div>
        {readerAutoReadEnabled ? (
          <div className="space-y-3 px-3 pt-1 pb-3">
            <ReaderRangeSetting
              label="竖向步进"
              value={readerAutoReadStripDistancePercent}
              min={READER_AUTO_READ_STRIP_DISTANCE_RANGE[0]}
              max={READER_AUTO_READ_STRIP_DISTANCE_RANGE[1]}
              step={5}
              suffix="%"
              onChange={setReaderAutoReadStripDistancePercent}
            />
            <ReaderRangeSetting
              label="竖向间隔"
              value={readerAutoReadStripIntervalMs}
              min={READER_AUTO_READ_STRIP_INTERVAL_RANGE[0]}
              max={READER_AUTO_READ_STRIP_INTERVAL_RANGE[1]}
              step={100}
              suffix="ms"
              onChange={setReaderAutoReadStripIntervalMs}
            />
            <ReaderRangeSetting
              label="单页间隔"
              value={readerAutoReadPageIntervalMs}
              min={READER_AUTO_READ_PAGE_INTERVAL_RANGE[0]}
              max={READER_AUTO_READ_PAGE_INTERVAL_RANGE[1]}
              step={200}
              suffix="ms"
              onChange={setReaderAutoReadPageIntervalMs}
            />
          </div>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

function ReaderRangeSetting({
  label,
  value,
  min,
  max,
  step,
  suffix,
  onChange
}: {
  label: string
  value: number
  min: number
  max: number
  step: number
  suffix: string
  onChange: (value: number) => void
}) {
  return (
    <label className="block space-y-1.5">
      <div className="flex items-center justify-between gap-3 text-xs">
        <span className="text-neutral-300">{label}</span>
        <span className="text-neutral-500 tabular-nums">
          {value}
          {suffix}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        className={cn(
          'h-4 w-full cursor-pointer appearance-none bg-transparent',
          '[&::-moz-range-track]:h-1 [&::-moz-range-track]:rounded-full [&::-moz-range-track]:bg-white/15',
          '[&::-moz-range-thumb]:size-3 [&::-moz-range-thumb]:rounded-full [&::-moz-range-thumb]:border-0 [&::-moz-range-thumb]:bg-neutral-100',
          '[&::-webkit-slider-runnable-track]:h-1 [&::-webkit-slider-runnable-track]:rounded-full [&::-webkit-slider-runnable-track]:bg-white/15',
          '[&::-webkit-slider-thumb]:mt-[-4px] [&::-webkit-slider-thumb]:size-3 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-neutral-100'
        )}
        onChange={event => onChange(Number(event.currentTarget.value))}
      />
    </label>
  )
}

function ReaderDirectionButton({
  selected,
  onClick,
  children
}: {
  selected: boolean
  onClick: () => void
  children: ReactNode
}) {
  return (
    <button
      type="button"
      className={cn(
        'h-8 rounded-md px-2 text-xs text-neutral-300 transition-colors hover:bg-white/10 hover:text-neutral-50 focus-visible:bg-white/10 focus-visible:text-neutral-50 focus-visible:outline-none',
        selected && 'bg-white/12 text-neutral-50 ring-1 ring-white/15'
      )}
      onClick={onClick}
    >
      {children}
    </button>
  )
}
