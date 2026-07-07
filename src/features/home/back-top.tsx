import { ChevronUpIcon } from 'lucide-react'
import { useEffect, useState } from 'react'

import { Button } from '@/components/ui/button'

export function BackTop() {
  const isVisible = useBackTopVisibility()

  if (!isVisible) {
    return null
  }

  return (
    <Button
      type="button"
      variant="outline"
      size="icon"
      aria-label="回到顶部"
      className="fixed right-8 bottom-8 z-50 bg-background/80 backdrop-blur"
      onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
    >
      <ChevronUpIcon className="size-4" />
    </Button>
  )
}

function useBackTopVisibility() {
  const [isVisible, setIsVisible] = useState(false)

  useEffect(() => {
    let frame = 0

    function updateVisibility() {
      cancelAnimationFrame(frame)
      frame = requestAnimationFrame(() => {
        setIsVisible(window.scrollY > 480)
      })
    }

    updateVisibility()
    window.addEventListener('scroll', updateVisibility, { passive: true })

    return () => {
      cancelAnimationFrame(frame)
      window.removeEventListener('scroll', updateVisibility)
    }
  }, [])

  return isVisible
}
