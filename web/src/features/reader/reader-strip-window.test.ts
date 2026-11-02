import { describe, expect, test } from 'vitest'

import { getStripAnchorOffset, resolveStripTrackedIndex } from './reader-strip-window'

describe('resolveStripTrackedIndex', () => {
  test('short prefetched content does not advance progress to the final chapter', () => {
    expect(
      resolveStripTrackedIndex({
        candidateIndex: 0,
        pageCount: 3,
        scrollTop: 0,
        scrollHeight: 800,
        clientHeight: 800
      })
    ).toBe(0)
  })
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

describe('stable strip anchors', () => {
  test('keeps the same point inside a page after prepending or recycling chapters', () => {
    const anchor = { key: 'chapter-2:3', offset: 175 }
    expect(getStripAnchorOffset(anchor, [{ key: anchor.key, start: 2400 }])).toBe(2575)
    expect(getStripAnchorOffset(anchor, [{ key: anchor.key, start: 4400 }])).toBe(4575)
    expect(getStripAnchorOffset(anchor, [{ key: anchor.key, start: 400 }])).toBe(575)
  })

  test('preserves a page below the viewport top and ignores an absent anchor', () => {
    expect(getStripAnchorOffset({ key: 'next', offset: -120 }, [{ key: 'next', start: 900 }])).toBe(
      780
    )
    expect(
      getStripAnchorOffset({ key: 'old', offset: 0 }, [{ key: 'next', start: 900 }])
    ).toBeNull()
  })
})
