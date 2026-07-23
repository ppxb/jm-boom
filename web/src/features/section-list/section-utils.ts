import type { HomeSectionListMode } from '@/lib/api/home'
import { currentChinaWeekday, parseStringSearch } from '@/lib/utils'
import {
  defaultRankingCategory,
  rankingCategoryOptions,
  RANKING_ORDER_OPTIONS,
  WEEK_CATEGORY_OPTIONS,
  WEEK_OPTIONS
} from '@/lib/filters'

export function parseRankingCategory(value: unknown, rankTag = '') {
  const fallback = defaultRankingCategory(rankTag)
  const category = parseStringSearch(value, fallback)

  return rankingCategoryOptions(rankTag).some(option => option.value === category)
    ? category
    : fallback
}

export function parseListCategory(mode: HomeSectionListMode, rankTag: string, value: unknown) {
  if (mode === 'ranking') {
    return parseRankingCategory(value, rankTag)
  }

  const category = parseStringSearch(value, 'all')

  if (mode === 'weekly') {
    return WEEK_CATEGORY_OPTIONS.some(option => option.value === category) ? category : 'all'
  }

  return 'all'
}

export function parseListWeek(value: unknown) {
  const week = parseStringSearch(value, String(currentChinaWeekday()))

  return WEEK_OPTIONS.some(option => option.value === week) ? week : String(currentChinaWeekday())
}

export function parseListOrder(value: unknown) {
  const order = parseStringSearch(value, 'new')

  return RANKING_ORDER_OPTIONS.some((option: { value: string }) => option.value === order)
    ? order
    : 'new'
}

export function isHomeSectionListMode(value: unknown): value is HomeSectionListMode {
  return value === 'promote' || value === 'weekly' || value === 'latest' || value === 'ranking'
}

export function sectionModeDescription(mode: HomeSectionListMode) {
  switch (mode) {
    case 'weekly':
      return '按星期和分类筛选连载更新'
    case 'latest':
      return '最新更新内容'
    case 'ranking':
      return '按分类和排序筛选更新内容'
    case 'promote':
    default:
      return '精选分组作品'
  }
}

export function sectionModeTitle(mode: HomeSectionListMode) {
  switch (mode) {
    case 'weekly':
      return '每周连载更新'
    case 'latest':
      return '最新'
    case 'ranking':
      return '分类更新'
    case 'promote':
    default:
      return '推荐'
  }
}
