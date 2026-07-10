/**
 * Centralized constants for the application
 */

// Time constants
export const TIME = {
  SECOND: 1000,
  MINUTE: 60 * 1000,
  HOUR: 60 * 60 * 1000
} as const

// Cache configuration
export const CACHE = {
  // List queries (home, ranking, weekly, etc.)
  LIST_STALE_TIME: 30 * TIME.MINUTE,
  LIST_GC_TIME: 6 * TIME.HOUR,

  // Comic detail page
  DETAIL_STALE_TIME: 10 * TIME.MINUTE,
  DETAIL_GC_TIME: TIME.HOUR,

  // Comments
  COMMENTS_STALE_TIME: 2 * TIME.MINUTE,
  COMMENTS_GC_TIME: 10 * TIME.MINUTE,

  // Reader
  READER_STALE_TIME: TIME.HOUR,
  READER_GC_TIME: 2 * TIME.HOUR,

  // Long-lived filters
  FILTERS_STALE_TIME: 12 * TIME.HOUR,
  FILTERS_GC_TIME: 24 * TIME.HOUR
} as const

// Reader configuration
export const READER = {
  PREFETCH_RADIUS: 2,
  STRIP_SCROLL_THRESHOLD: 24,
  DEFAULT_CHAPTER_TITLE: '正文'
} as const

// UI configuration
export const UI = {
  CHAPTER_PAGE_SIZE: 10,
  COMMENT_SKELETON_COUNT: 6,
  SHOW_COVER_MASK: true
} as const

// Download configuration
export const DOWNLOAD = {
  PROGRESS_PERSIST_INTERVAL: TIME.SECOND
} as const
