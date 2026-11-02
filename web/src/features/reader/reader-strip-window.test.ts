import { describe, expect, test } from 'vitest'

import {
  createStripItems,
  getStripAnchorOffset,
  resolveStripTrackedIndex
} from './reader-strip-window'
import type { ReaderBoundary, ReaderPageItem } from './types'

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

describe('strip chapter transitions', () => {
  const titles = new Map([
    ['a', 'Chapter A'],
    ['b', 'Chapter B']
  ])
  const end: ReaderBoundary = { chapter: null, loading: false, error: null }
  const page = (chapterId: string, index: number): ReaderPageItem => ({
    chapterId,
    index,
    localIndex: index,
    key: chapterId + ':' + index,
    path: '/' + chapterId + '/' + index,
    name: String(index)
  })

  test('always inserts full chapter information without changing image page indices', () => {
    const items = createStripItems([page('a', 0), page('a', 1), page('b', 0)], titles, end, end)
    expect(items.filter(item => !item.transition).map(item => item.pageIndex)).toEqual([0, 1, 2])
    const card = items.find(item => item.key === 'chapter:a>b')!
    expect(card.transition?.currentChapter.title).toBe('Chapter A')
    expect(card.transition?.boundary.chapter?.title).toBe('Chapter B')
    expect(items[items.length - 1].transition?.boundary.chapter).toBeNull()
  })

  test('keeps the boundary identity when a pending chapter is appended or prepended', () => {
    const before = createStripItems([page('a', 0)], titles, end, {
      chapter: { id: 'b', title: 'Chapter B' },
      loading: true,
      error: null
    })
    const after = createStripItems([page('a', 0), page('b', 0)], titles, end, end)
    expect(before[1].key).toBe(after[1].key)
    const reverse = createStripItems(
      [page('b', 0)],
      titles,
      {
        chapter: { id: 'a', title: 'Chapter A' },
        loading: true,
        error: null
      },
      end
    )
    expect(reverse[0].key).toBe(after[1].key)
    expect(reverse[0].transition?.previous).toBe(true)
  })
})
