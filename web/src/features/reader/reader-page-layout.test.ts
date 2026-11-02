import { describe, expect, test } from 'vitest'
import { readerSpread, readerSpreadStarts } from './reader-page-layout'
import type { ReaderPageItem } from './types'

function page(chapterId: string, localIndex: number, wide = false): ReaderPageItem {
  return {
    key: chapterId + ':' + localIndex,
    chapterId,
    localIndex,
    index: localIndex,
    name: String(localIndex),
    path: '/' + chapterId + '/' + localIndex,
    width: wide ? 1200 : 800,
    height: wide ? 800 : 1200
  }
}

describe('reader page layout', () => {
  test('keeps spreads and odd chapter endings on their own screens', () => {
    const pages = [
      page('a', 0),
      page('a', 1),
      page('a', 2, true),
      page('a', 3),
      page('b', 0),
      page('b', 1)
    ]
    expect(readerSpreadStarts(pages, true)).toEqual([0, 2, 3, 4])
    expect(readerSpread(pages, 3, true)).toEqual({ start: 3, end: 4, previous: 2 })
    expect(readerSpread(pages, 5, true)).toEqual({ start: 4, end: 6, previous: 3 })
  })

  test('a mode change retains the selected page inside the displayed pair', () => {
    const pages = [page('a', 0), page('a', 1), page('a', 2), page('a', 3)]
    expect(readerSpread(pages, 1, true)).toEqual({ start: 0, end: 2, previous: -1 })
    expect(readerSpread(pages, 1, false)).toEqual({ start: 1, end: 2, previous: 0 })
  })
})
