import { describe, expect, test } from 'vitest'

import { toReaderChapterSearch } from './reader-chapter-link'

describe('toReaderChapterSearch', () => {
  test('always starts a directly selected chapter from its first page', () => {
    expect(toReaderChapterSearch({ albumId: '123' })).toEqual({ albumId: '123', page: 1 })
  })
})
