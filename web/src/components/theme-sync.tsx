import { useEffect } from 'react'

import { applyTheme, isTheme } from '@/lib/theme'
import { useThemeStore } from '@/stores/theme-store'

export function ThemeSync() {
  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const syncSystemTheme = () => {
      if (useThemeStore.getState().theme === 'system') {
        useThemeStore.setState({ resolvedTheme: applyTheme('system') })
      }
    }
    const syncStoredTheme = (event: StorageEvent) => {
      if (event.key !== 'theme' && event.key !== null) return
      const theme = isTheme(event.newValue) ? event.newValue : 'system'
      useThemeStore.setState({ theme, resolvedTheme: applyTheme(theme) })
    }
    syncSystemTheme()
    media.addEventListener('change', syncSystemTheme)
    window.addEventListener('storage', syncStoredTheme)
    return () => {
      media.removeEventListener('change', syncSystemTheme)
      window.removeEventListener('storage', syncStoredTheme)
    }
  }, [])

  return null
}
