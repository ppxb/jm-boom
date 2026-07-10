import { useCallback, useEffect, useRef, type RefObject } from 'react'

import type { ReaderReadMode } from '@/stores/settings-store'
import { useSettingsStore } from '@/stores/settings-store'

const STRIP_BOTTOM_THRESHOLD_PX = 2

export function useReaderAutoRead({
  readMode,
  pageStep,
  pageCount,
  currentIndex,
  controlsVisible,
  canAdvance,
  stripScrollRef,
  onNextPage
}: {
  readMode: ReaderReadMode
  pageStep: number
  pageCount: number
  currentIndex: number
  controlsVisible: boolean
  canAdvance: boolean
  stripScrollRef: RefObject<HTMLDivElement | null>
  onNextPage: () => void
}) {
  const enabled = useSettingsStore(state => state.readerAutoReadEnabled)
  const stripIntervalMs = useSettingsStore(state => state.readerAutoReadStripIntervalMs)
  const pageIntervalMs = useSettingsStore(state => state.readerAutoReadPageIntervalMs)
  const stripDistancePercent = useSettingsStore(state => state.readerAutoReadStripDistancePercent)
  const wasEnabledRef = useRef(enabled)
  const wasControlsVisibleRef = useRef(controlsVisible)
  const wasCanAdvanceRef = useRef(canAdvance)
  const pendingImmediateAdvanceRef = useRef(enabled)
  const isStripMode = readMode === 'strip'
  const intervalMs = isStripMode ? stripIntervalMs : pageIntervalMs
  const isLastPage = pageCount > 0 && currentIndex >= pageCount - pageStep
  const isActive = enabled && !controlsVisible && canAdvance && pageCount > 0

  useEffect(() => {
    if (enabled && !wasEnabledRef.current) {
      pendingImmediateAdvanceRef.current = true
    }

    if (!enabled) {
      pendingImmediateAdvanceRef.current = false
    }

    if (enabled && wasControlsVisibleRef.current && !controlsVisible) {
      pendingImmediateAdvanceRef.current = true
    }

    if (enabled && !wasCanAdvanceRef.current && canAdvance) {
      pendingImmediateAdvanceRef.current = true
    }

    wasEnabledRef.current = enabled
    wasControlsVisibleRef.current = controlsVisible
    wasCanAdvanceRef.current = canAdvance
  }, [canAdvance, controlsVisible, enabled])

  const advanceStrip = useCallback(() => {
    const container = stripScrollRef.current

    if (!container) {
      return
    }

    const maxScrollTop = Math.max(container.scrollHeight - container.clientHeight, 0)

    if (maxScrollTop <= 0 || container.scrollTop >= maxScrollTop - STRIP_BOTTOM_THRESHOLD_PX) {
      return
    }

    container.scrollBy({
      top: container.clientHeight * (stripDistancePercent / 100),
      behavior: 'smooth'
    })
  }, [stripDistancePercent, stripScrollRef])

  const advance = useCallback(() => {
    if (!enabled || controlsVisible || !canAdvance || pageCount <= 0) {
      return
    }

    if (isStripMode) {
      advanceStrip()
      return
    }

    if (!isLastPage) {
      onNextPage()
    }
  }, [
    advanceStrip,
    canAdvance,
    controlsVisible,
    enabled,
    isLastPage,
    isStripMode,
    onNextPage,
    pageCount
  ])

  useEffect(() => {
    if (!isActive || !pendingImmediateAdvanceRef.current) {
      return
    }

    pendingImmediateAdvanceRef.current = false
    const frame = window.requestAnimationFrame(advance)

    return () => window.cancelAnimationFrame(frame)
  }, [advance, isActive])

  useEffect(() => {
    if (!enabled || !canAdvance || pageCount <= 0) {
      return
    }

    const timer = window.setInterval(advance, intervalMs)

    return () => window.clearInterval(timer)
  }, [advance, canAdvance, enabled, intervalMs, pageCount])
}
