import { describe, expect, test } from 'bun:test'

import { resolveStripTrackedIndex, shouldAdvanceStripChapter } from './reader-strip-window'

describe('resolveStripTrackedIndex', () => {
  test('uses the candidate page before reaching the real scroll bottom', () => {
    expect(
      resolveStripTrackedIndex({
        candidateIndex: 8,
        pageCount: 10,
        scrollTop: 1175,
        scrollHeight: 2000,
        clientHeight: 800
      })
    ).toBe(8)
  })

  test('tracks the final page when the strip reaches the real scroll bottom', () => {
    expect(
      resolveStripTrackedIndex({
        candidateIndex: 8,
        pageCount: 10,
        scrollTop: 1176,
        scrollHeight: 2000,
        clientHeight: 800
      })
    ).toBe(9)
  })
})

describe('shouldAdvanceStripChapter', () => {
  test('requires a continued upward swipe after reaching the bottom', () => {
    expect(shouldAdvanceStripChapter(300, 253)).toBe(false)
    expect(shouldAdvanceStripChapter(300, 252)).toBe(true)
  })
})
