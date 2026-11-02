export type Theme = 'light' | 'dark' | 'system'
export type ResolvedTheme = Exclude<Theme, 'system'>

let transitionFrame = 0

// Keep this function self-contained: Vite inlines it before the first paint.
export function initializeTheme(): Theme {
  let theme: Theme = 'system'
  try {
    const stored = localStorage.getItem('theme')
    if (stored === 'light' || stored === 'dark') theme = stored
  } catch {
    // The theme still works when browser storage is unavailable.
  }
  const dark =
    theme === 'dark' ||
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  const root = document.documentElement
  root.classList.toggle('dark', dark)
  root.classList.toggle('light', !dark)
  root.style.colorScheme = dark ? 'dark' : 'light'
  return theme
}

export function applyTheme(theme: Theme): ResolvedTheme {
  const resolved =
    theme === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
      : theme
  const root = document.documentElement
  cancelAnimationFrame(transitionFrame)
  root.classList.add('disable-transitions')
  root.classList.toggle('dark', resolved === 'dark')
  root.classList.toggle('light', resolved === 'light')
  root.style.colorScheme = resolved
  // Keep transitions disabled through the first paint with the new colors.
  transitionFrame = requestAnimationFrame(() => {
    transitionFrame = requestAnimationFrame(() => {
      root.classList.remove('disable-transitions')
      transitionFrame = 0
    })
  })
  return resolved
}

export function isTheme(value: unknown): value is Theme {
  return value === 'light' || value === 'dark' || value === 'system'
}
