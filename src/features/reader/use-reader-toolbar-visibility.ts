import { useCallback, useState } from 'react'

export function useReaderToolbarVisibility() {
  const [isVisible, setIsVisible] = useState(true)

  const hide = useCallback(() => {
    setIsVisible(false)
  }, [])

  const toggle = useCallback(() => {
    setIsVisible(visible => !visible)
  }, [])

  return { isVisible, toggle, hide }
}
