import { describe, expect, test } from 'bun:test'

import { shouldPrefetchNextChapter } from './use-next-chapter-prefetch'

describe('shouldPrefetchNextChapter', () => {
  test('prefetches when the current page reaches the configured progress threshold', () => {
    expect(shouldPrefetchNextChapter(79, 100)).toBe(true)
    expect(shouldPrefetchNextChapter(78, 100)).toBe(false)
  })

  test('prefetches when only the configured lookahead pages remain', () => {
    expect(shouldPrefetchNextChapter(93, 100)).toBe(true)
  })

  test('accepts the strip scroll threshold as a mobile fallback', () => {
    expect(shouldPrefetchNextChapter(20, 100, true)).toBe(true)
  })
})
