import { describe, expect, test } from 'bun:test'

import { parseListOrder, parseRankingCategory } from './section-utils'

describe('parseRankingCategory', () => {
  test('accepts valid categories and falls back to the default category', () => {
    expect(parseRankingCategory('single')).toBe('single')
    expect(parseRankingCategory('invalid')).toBe('latest')
    expect(parseRankingCategory(undefined)).toBe('latest')
  })

  test('uses the rank tag options and fallback', () => {
    expect(parseRankingCategory('hanman_chinese', 'hanManTypeMap')).toBe('hanman_chinese')
    expect(parseRankingCategory('single', 'hanManTypeMap')).toBe('hanman')
  })
})

describe('parseListOrder', () => {
  test('accepts valid orders and falls back to newest', () => {
    expect(parseListOrder('mv_w')).toBe('mv_w')
    expect(parseListOrder('invalid')).toBe('new')
    expect(parseListOrder(undefined)).toBe('new')
  })
})
