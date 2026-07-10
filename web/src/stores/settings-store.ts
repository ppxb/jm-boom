import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export const FALLBACK_API_ENDPOINTS = ['https://www.cdnhjk.net', 'https://www.cdnhth.club'] as const

export const READER_CACHE_LIMITS_MB = [128, 256, 512, 1024, 2048] as const
export const PROXY_MODES = ['off', 'http', 'socks5'] as const
export const READER_READ_MODES = ['single', 'strip'] as const
export const READER_PAGE_DIRECTIONS = ['ltr', 'rtl'] as const
export const READER_AUTO_READ_STRIP_DISTANCE_RANGE = [10, 100] as const
export const READER_AUTO_READ_STRIP_INTERVAL_RANGE = [300, 5000] as const
export const READER_AUTO_READ_PAGE_INTERVAL_RANGE = [800, 10000] as const

export type ApiEndpoint = string
export type ReaderCacheLimitMb = (typeof READER_CACHE_LIMITS_MB)[number]
export type ProxyMode = (typeof PROXY_MODES)[number]
export type ReaderReadMode = (typeof READER_READ_MODES)[number]
export type ReaderPageDirection = (typeof READER_PAGE_DIRECTIONS)[number]

type SettingsState = {
  api: ApiEndpoint
  readerCacheLimitMb: ReaderCacheLimitMb
  readerReadMode: ReaderReadMode
  readerPageDirection: ReaderPageDirection
  readerDoublePageMode: boolean
  readerAutoReadEnabled: boolean
  readerAutoReadStripIntervalMs: number
  readerAutoReadPageIntervalMs: number
  readerAutoReadStripDistancePercent: number
  proxyMode: ProxyMode
  proxyHost: string
  proxyPort: number
  hideCovers: boolean
  nsfwWarningDismissed: boolean
  setApi: (api: string) => void
  setReaderCacheLimitMb: (readerCacheLimitMb: number) => void
  setReaderReadMode: (readerReadMode: string) => void
  setReaderPageDirection: (readerPageDirection: string) => void
  setReaderDoublePageMode: (readerDoublePageMode: boolean) => void
  setReaderAutoReadEnabled: (readerAutoReadEnabled: boolean) => void
  setReaderAutoReadStripIntervalMs: (readerAutoReadStripIntervalMs: number) => void
  setReaderAutoReadPageIntervalMs: (readerAutoReadPageIntervalMs: number) => void
  setReaderAutoReadStripDistancePercent: (readerAutoReadStripDistancePercent: number) => void
  setProxyMode: (proxyMode: string) => void
  setProxyHost: (proxyHost: string) => void
  setProxyPort: (proxyPort: number) => void
  setHideCovers: (hideCovers: boolean) => void
  dismissNsfwWarning: () => void
  reset: () => void
}

const DEFAULT_SETTINGS = {
  api: FALLBACK_API_ENDPOINTS[0],
  readerCacheLimitMb: 512,
  readerReadMode: READER_READ_MODES[0],
  readerPageDirection: READER_PAGE_DIRECTIONS[0],
  readerDoublePageMode: false,
  readerAutoReadEnabled: false,
  readerAutoReadStripIntervalMs: 1600,
  readerAutoReadPageIntervalMs: 3000,
  readerAutoReadStripDistancePercent: 72,
  proxyMode: PROXY_MODES[0],
  proxyHost: '127.0.0.1',
  proxyPort: 7890,
  hideCovers: true,
  nsfwWarningDismissed: false
} satisfies Pick<
  SettingsState,
  | 'api'
  | 'readerCacheLimitMb'
  | 'readerReadMode'
  | 'readerPageDirection'
  | 'readerDoublePageMode'
  | 'readerAutoReadEnabled'
  | 'readerAutoReadStripIntervalMs'
  | 'readerAutoReadPageIntervalMs'
  | 'readerAutoReadStripDistancePercent'
  | 'proxyMode'
  | 'proxyHost'
  | 'proxyPort'
  | 'hideCovers'
  | 'nsfwWarningDismissed'
>

export const useSettingsStore = create<SettingsState>()(
  persist(
    set => ({
      ...DEFAULT_SETTINGS,
      setApi: api => {
        set({
          api: normalizeApiEndpoint(api) || DEFAULT_SETTINGS.api
        })
      },
      setReaderCacheLimitMb: readerCacheLimitMb => {
        set({
          readerCacheLimitMb: isReaderCacheLimitMb(readerCacheLimitMb)
            ? readerCacheLimitMb
            : DEFAULT_SETTINGS.readerCacheLimitMb
        })
      },
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
      setReaderDoublePageMode: readerDoublePageMode => {
        set({ readerDoublePageMode })
      },
      setReaderAutoReadEnabled: readerAutoReadEnabled => {
        set({ readerAutoReadEnabled })
      },
      setReaderAutoReadStripIntervalMs: readerAutoReadStripIntervalMs => {
        set({
          readerAutoReadStripIntervalMs: clampNumber(
            readerAutoReadStripIntervalMs,
            READER_AUTO_READ_STRIP_INTERVAL_RANGE[0],
            READER_AUTO_READ_STRIP_INTERVAL_RANGE[1],
            DEFAULT_SETTINGS.readerAutoReadStripIntervalMs
          )
        })
      },
      setReaderAutoReadPageIntervalMs: readerAutoReadPageIntervalMs => {
        set({
          readerAutoReadPageIntervalMs: clampNumber(
            readerAutoReadPageIntervalMs,
            READER_AUTO_READ_PAGE_INTERVAL_RANGE[0],
            READER_AUTO_READ_PAGE_INTERVAL_RANGE[1],
            DEFAULT_SETTINGS.readerAutoReadPageIntervalMs
          )
        })
      },
      setReaderAutoReadStripDistancePercent: readerAutoReadStripDistancePercent => {
        set({
          readerAutoReadStripDistancePercent: clampNumber(
            readerAutoReadStripDistancePercent,
            READER_AUTO_READ_STRIP_DISTANCE_RANGE[0],
            READER_AUTO_READ_STRIP_DISTANCE_RANGE[1],
            DEFAULT_SETTINGS.readerAutoReadStripDistancePercent
          )
        })
      },
      setProxyMode: proxyMode => {
        set({
          proxyMode: isProxyMode(proxyMode) ? proxyMode : DEFAULT_SETTINGS.proxyMode
        })
      },
      setProxyHost: proxyHost => {
        set({
          proxyHost: proxyHost.trim()
        })
      },
      setProxyPort: proxyPort => {
        set({
          proxyPort: isProxyPort(proxyPort) ? proxyPort : DEFAULT_SETTINGS.proxyPort
        })
      },
      setHideCovers: hideCovers => {
        set({ hideCovers })
      },
      dismissNsfwWarning: () => {
        set({ nsfwWarningDismissed: true })
      },
      reset: () => {
        set(DEFAULT_SETTINGS)
      }
    }),
    {
      name: 'jm-boom-settings',
      partialize: state => ({
        api: state.api,
        readerCacheLimitMb: state.readerCacheLimitMb,
        readerReadMode: state.readerReadMode,
        readerPageDirection: state.readerPageDirection,
        readerDoublePageMode: state.readerDoublePageMode,
        readerAutoReadEnabled: state.readerAutoReadEnabled,
        readerAutoReadStripIntervalMs: state.readerAutoReadStripIntervalMs,
        readerAutoReadPageIntervalMs: state.readerAutoReadPageIntervalMs,
        readerAutoReadStripDistancePercent: state.readerAutoReadStripDistancePercent,
        proxyMode: state.proxyMode,
        proxyHost: state.proxyHost,
        proxyPort: state.proxyPort,
        hideCovers: state.hideCovers,
        nsfwWarningDismissed: state.nsfwWarningDismissed
      })
    }
  )
)

function normalizeApiEndpoint(value: string) {
  const endpoint = value.trim().replace(/\/+$/, '')

  if (!endpoint) {
    return ''
  }

  return /^https?:\/\//i.test(endpoint) ? endpoint : `https://${endpoint}`
}

function isReaderCacheLimitMb(value: number): value is ReaderCacheLimitMb {
  return READER_CACHE_LIMITS_MB.some(limit => limit === value)
}

function isProxyMode(value: string): value is ProxyMode {
  return PROXY_MODES.some(mode => mode === value)
}

function isReaderReadMode(value: string): value is ReaderReadMode {
  return READER_READ_MODES.some(mode => mode === value)
}

function isReaderPageDirection(value: string): value is ReaderPageDirection {
  return READER_PAGE_DIRECTIONS.some(direction => direction === value)
}

function isProxyPort(value: number) {
  return Number.isInteger(value) && value > 0 && value <= 65535
}

function clampNumber(value: number, min: number, max: number, fallback: number) {
  if (!Number.isFinite(value)) {
    return fallback
  }

  return Math.min(Math.max(Math.round(value), min), max)
}
