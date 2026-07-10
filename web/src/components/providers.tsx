import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from 'next-themes'
import { ReactNode, useEffect, useRef } from 'react'
import { toast, Toaster } from 'sonner'

import { checkAppUpdate, configureNetworkProxy } from '@/lib/api/setting'
import { hasTauriRuntime } from '@/lib/api/tauri'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { useSettingsStore } from '@/stores/settings-store'
import { TooltipProvider } from './ui/tooltip'

const DEFAULT_QUERY_RETRY_COUNT = 2

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: CACHE.LIST_STALE_TIME,
      gcTime: CACHE.LIST_GC_TIME,
      refetchOnMount: false,
      refetchOnWindowFocus: false,
      retry: (failureCount, error) =>
        failureCount < DEFAULT_QUERY_RETRY_COUNT && isRetryableQueryError(error)
    }
  }
})

export function Providers({ children }: { children: ReactNode }) {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider attribute="class" enableSystem={true} disableTransitionOnChange>
        <TooltipProvider>{children}</TooltipProvider>
        <StartupUpdateCheck />
        <Toaster
          toastOptions={{
            classNames: {
              toast: 'font-sans'
            }
          }}
        />
      </ThemeProvider>
    </QueryClientProvider>
  )
}

function StartupUpdateCheck() {
  const proxyMode = useSettingsStore(state => state.proxyMode)
  const proxyHost = useSettingsStore(state => state.proxyHost)
  const proxyPort = useSettingsStore(state => state.proxyPort)
  const hasCheckedRef = useRef(false)

  useEffect(() => {
    if (hasCheckedRef.current || !hasTauriRuntime()) {
      return
    }

    hasCheckedRef.current = true
    const timer = window.setTimeout(() => {
      void (async () => {
        await configureNetworkProxy({ mode: proxyMode, host: proxyHost, port: proxyPort })
        const update = await checkAppUpdate()

        queryClient.setQueryData(queryKeys.appUpdate(), update)

        if (update.currentVersion) {
          queryClient.setQueryData(queryKeys.appVersion(), update.currentVersion)
        }

        if (update.available && update.version) {
          toast.success(`发现新版本 ${update.version}`)
        }
      })().catch(error => {
        if (import.meta.env.DEV) {
          console.debug('Startup update check failed', error)
        }
      })
    }, 1500)

    return () => window.clearTimeout(timer)
  }, [proxyHost, proxyMode, proxyPort])

  return null
}

function isRetryableQueryError(error: unknown) {
  if (
    typeof error === 'object' &&
    error !== null &&
    'retryable' in error &&
    typeof error.retryable === 'boolean'
  ) {
    return error.retryable
  }

  return true
}
