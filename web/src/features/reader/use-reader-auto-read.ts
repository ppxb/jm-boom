import { useEffect, type RefObject } from 'react'
import { useSettingsStore, type ReaderReadMode } from '@/stores/settings-store'

export function useReaderAutoRead({
  readMode,
  pageCount,
  controlsVisible,
  canAdvance,
  stripScrollRef,
  onNextPage,
  onStripEnd
}: {
  readMode: ReaderReadMode
  pageCount: number
  controlsVisible: boolean
  canAdvance: boolean
  stripScrollRef: RefObject<HTMLDivElement | null>
  onNextPage: () => void
  onStripEnd: () => void
}) {
  const enabled = useSettingsStore(state => state.readerAutoReadEnabled)
  const stripInterval = useSettingsStore(state => state.readerAutoReadStripIntervalMs)
  const pageInterval = useSettingsStore(state => state.readerAutoReadPageIntervalMs)
  const distance = useSettingsStore(state => state.readerAutoReadStripDistancePercent)

  useEffect(() => {
    if (!enabled || controlsVisible || !canAdvance || pageCount <= 0) return
    const advance = () => {
      if (readMode !== 'strip') {
        onNextPage()
        return
      }
      const container = stripScrollRef.current
      if (!container) return
      if (container.scrollHeight - container.clientHeight - container.scrollTop <= 2) {
        onStripEnd()
      } else {
        container.scrollBy({ top: (container.clientHeight * distance) / 100, behavior: 'smooth' })
      }
    }
    const timer = window.setInterval(advance, readMode === 'strip' ? stripInterval : pageInterval)
    return () => window.clearInterval(timer)
  }, [
    canAdvance,
    controlsVisible,
    distance,
    enabled,
    onNextPage,
    onStripEnd,
    pageCount,
    pageInterval,
    readMode,
    stripInterval,
    stripScrollRef
  ])
}
