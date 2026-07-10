import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ThemeProvider } from 'next-themes'
import { ReactNode } from 'react'
import { Toaster } from 'sonner'

import { CACHE } from '@/lib/constants'
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
