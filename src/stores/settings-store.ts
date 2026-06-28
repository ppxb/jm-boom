import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export const FALLBACK_API_ENDPOINTS = [
  'https://www.cdnhth.club',
  'https://www.cdnhjk.net'
] as const

export const READER_CACHE_LIMITS_MB = [128, 256, 512, 1024, 2048] as const
export const PROXY_MODES = ['off', 'http', 'socks5'] as const

export type ApiEndpoint = string
export type ReaderCacheLimitMb = (typeof READER_CACHE_LIMITS_MB)[number]
export type ProxyMode = (typeof PROXY_MODES)[number]

type SettingsState = {
  api: ApiEndpoint
  readerCacheLimitMb: ReaderCacheLimitMb
  proxyMode: ProxyMode
  proxyHost: string
  proxyPort: number
  hideCovers: boolean
  setApi: (api: string) => void
  setReaderCacheLimitMb: (readerCacheLimitMb: number) => void
  setProxyMode: (proxyMode: string) => void
  setProxyHost: (proxyHost: string) => void
  setProxyPort: (proxyPort: number) => void
  setHideCovers: (hideCovers: boolean) => void
  reset: () => void
}

const DEFAULT_SETTINGS = {
  api: FALLBACK_API_ENDPOINTS[0],
  readerCacheLimitMb: 512,
  proxyMode: PROXY_MODES[0],
  proxyHost: '127.0.0.1',
  proxyPort: 7890,
  hideCovers: true
} satisfies Pick<
  SettingsState,
  | 'api'
  | 'readerCacheLimitMb'
  | 'proxyMode'
  | 'proxyHost'
  | 'proxyPort'
  | 'hideCovers'
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
      reset: () => {
        set(DEFAULT_SETTINGS)
      }
    }),
    {
      name: 'jm-boom-settings',
      partialize: state => ({
        api: state.api,
        readerCacheLimitMb: state.readerCacheLimitMb,
        proxyMode: state.proxyMode,
        proxyHost: state.proxyHost,
        proxyPort: state.proxyPort,
        hideCovers: state.hideCovers
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

function isProxyPort(value: number) {
  return Number.isInteger(value) && value > 0 && value <= 65535
}
