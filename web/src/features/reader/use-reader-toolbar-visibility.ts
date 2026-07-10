import { useCallback, useState } from 'react'

export function useReaderToolbarVisibility(initialVisible = true) {
  const [isVisible, setIsVisible] = useState(initialVisible)

  const hide = useCallback(() => {
    setIsVisible(false)
  }, [])

  const toggle = useCallback(() => {
    setIsVisible(visible => !visible)
  }, [])

  return { isVisible, toggle, hide }
}
