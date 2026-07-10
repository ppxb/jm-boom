import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export const READER_READ_MODES = ['single', 'strip'] as const
export const READER_PAGE_DIRECTIONS = ['ltr', 'rtl'] as const
export const READER_AUTO_READ_STRIP_DISTANCE_RANGE = [10, 100] as const
export const READER_AUTO_READ_STRIP_INTERVAL_RANGE = [300, 5000] as const
export const READER_AUTO_READ_PAGE_INTERVAL_RANGE = [800, 10000] as const

export type ReaderReadMode = (typeof READER_READ_MODES)[number]
export type ReaderPageDirection = (typeof READER_PAGE_DIRECTIONS)[number]

type SettingsState = {
  readerReadMode: ReaderReadMode
  readerPageDirection: ReaderPageDirection
  readerDoublePageMode: boolean
  readerAutoReadEnabled: boolean
  readerAutoReadStripIntervalMs: number
  readerAutoReadPageIntervalMs: number
  readerAutoReadStripDistancePercent: number
  hideCovers: boolean
  nsfwWarningDismissed: boolean
  setReaderReadMode: (readerReadMode: string) => void
  setReaderPageDirection: (readerPageDirection: string) => void
  setReaderDoublePageMode: (readerDoublePageMode: boolean) => void
  setReaderAutoReadEnabled: (readerAutoReadEnabled: boolean) => void
  setReaderAutoReadStripIntervalMs: (readerAutoReadStripIntervalMs: number) => void
  setReaderAutoReadPageIntervalMs: (readerAutoReadPageIntervalMs: number) => void
  setReaderAutoReadStripDistancePercent: (readerAutoReadStripDistancePercent: number) => void
  setHideCovers: (hideCovers: boolean) => void
  dismissNsfwWarning: () => void
  reset: () => void
}

const DEFAULT_SETTINGS = {
  readerReadMode: READER_READ_MODES[0],
  readerPageDirection: READER_PAGE_DIRECTIONS[0],
  readerDoublePageMode: false,
  readerAutoReadEnabled: false,
  readerAutoReadStripIntervalMs: 1600,
  readerAutoReadPageIntervalMs: 3000,
  readerAutoReadStripDistancePercent: 72,
  hideCovers: true,
  nsfwWarningDismissed: false
} satisfies Pick<
  SettingsState,
  | 'readerReadMode'
  | 'readerPageDirection'
  | 'readerDoublePageMode'
  | 'readerAutoReadEnabled'
  | 'readerAutoReadStripIntervalMs'
  | 'readerAutoReadPageIntervalMs'
  | 'readerAutoReadStripDistancePercent'
  | 'hideCovers'
  | 'nsfwWarningDismissed'
>

export const useSettingsStore = create<SettingsState>()(
  persist(
    set => ({
      ...DEFAULT_SETTINGS,
      setReaderReadMode: readerReadMode => {
        set({
          readerReadMode: isReaderReadMode(readerReadMode)
            ? readerReadMode
            : DEFAULT_SETTINGS.readerReadMode
        })
      },
      setReaderPageDirection: readerPageDirection => {
        set({
          readerPageDirection: isReaderPageDirection(readerPageDirection)
            ? readerPageDirection
            : DEFAULT_SETTINGS.readerPageDirection
        })
      },
      setReaderDoublePageMode: readerDoublePageMode => set({ readerDoublePageMode }),
      setReaderAutoReadEnabled: readerAutoReadEnabled => set({ readerAutoReadEnabled }),
      setReaderAutoReadStripIntervalMs: value => {
        set({
          readerAutoReadStripIntervalMs: clampNumber(
            value,
            READER_AUTO_READ_STRIP_INTERVAL_RANGE[0],
            READER_AUTO_READ_STRIP_INTERVAL_RANGE[1],
            DEFAULT_SETTINGS.readerAutoReadStripIntervalMs
          )
        })
      },
      setReaderAutoReadPageIntervalMs: value => {
        set({
          readerAutoReadPageIntervalMs: clampNumber(
            value,
            READER_AUTO_READ_PAGE_INTERVAL_RANGE[0],
            READER_AUTO_READ_PAGE_INTERVAL_RANGE[1],
            DEFAULT_SETTINGS.readerAutoReadPageIntervalMs
          )
        })
      },
      setReaderAutoReadStripDistancePercent: value => {
        set({
          readerAutoReadStripDistancePercent: clampNumber(
            value,
            READER_AUTO_READ_STRIP_DISTANCE_RANGE[0],
            READER_AUTO_READ_STRIP_DISTANCE_RANGE[1],
            DEFAULT_SETTINGS.readerAutoReadStripDistancePercent
          )
        })
      },
      setHideCovers: hideCovers => set({ hideCovers }),
      dismissNsfwWarning: () => set({ nsfwWarningDismissed: true }),
      reset: () => set(DEFAULT_SETTINGS)
    }),
    {
      name: 'jm-boom-settings',
      partialize: state => ({
        readerReadMode: state.readerReadMode,
        readerPageDirection: state.readerPageDirection,
        readerDoublePageMode: state.readerDoublePageMode,
        readerAutoReadEnabled: state.readerAutoReadEnabled,
        readerAutoReadStripIntervalMs: state.readerAutoReadStripIntervalMs,
        readerAutoReadPageIntervalMs: state.readerAutoReadPageIntervalMs,
        readerAutoReadStripDistancePercent: state.readerAutoReadStripDistancePercent,
        hideCovers: state.hideCovers,
        nsfwWarningDismissed: state.nsfwWarningDismissed
      })
    }
  )
)

function isReaderReadMode(value: string): value is ReaderReadMode {
  return READER_READ_MODES.some(mode => mode === value)
}

function isReaderPageDirection(value: string): value is ReaderPageDirection {
  return READER_PAGE_DIRECTIONS.some(direction => direction === value)
}

function clampNumber(value: number, min: number, max: number, fallback: number) {
  if (!Number.isFinite(value)) {
    return fallback
  }

  return Math.min(Math.max(Math.round(value), min), max)
}
