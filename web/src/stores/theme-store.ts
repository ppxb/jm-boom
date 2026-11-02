import { create } from 'zustand'

import { applyTheme, initializeTheme, type ResolvedTheme, type Theme } from '@/lib/theme'

type ThemeState = {
  theme: Theme
  resolvedTheme: ResolvedTheme
  setTheme: (theme: Theme) => void
}

const initialTheme = initializeTheme()

export const useThemeStore = create<ThemeState>(set => ({
  theme: initialTheme,
  resolvedTheme: document.documentElement.classList.contains('dark') ? 'dark' : 'light',
  setTheme: theme => {
    set({ theme, resolvedTheme: applyTheme(theme) })
    try {
      localStorage.setItem('theme', theme)
    } catch {
      // Keep applying the user's choice even if it cannot be persisted.
    }
  }
}))
