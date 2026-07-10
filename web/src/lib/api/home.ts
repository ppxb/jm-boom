import { tauriInvoke } from './tauri'

export type FeedComic = {
  id: string
  title: string
  author: string
  description: string
  image: string
  tags: string[]
  updatedAt?: number | null
}

export type HomeFeedSection = {
  id: string
  title: string
  slug: string
  type: string
  filterValue: string
  listMode: HomeSectionListMode | null
  rankTag: string
  items: FeedComic[]
}

export type HomeSectionListMode = 'promote' | 'weekly' | 'latest' | 'ranking'

export type HomeSectionListParams = {
  mode: HomeSectionListMode
  page?: number
  sectionId?: string | null
  sectionTitle?: string | null
  slug?: string | null
  type?: string | null
  filterValue?: string | null
  category?: string | null
  week?: string | null
  order?: string | null
  endpoint?: string | null
}

export type HomeSectionListResult = {
  endpoint: string
  mode: HomeSectionListMode
  page: number
  pageSize: number
  total: number
  hasMore: boolean
  title: string
  items: FeedComic[]
}

export type HomeFeedResult = {
  endpoint: string
  sections: HomeFeedSection[]
}

export type WeekCategory = {
  id: string
  time: string
  title: string
  label: string
}

export type WeekType = {
  id: string
  title: string
}

export type WeekFiltersResult = {
  endpoint: string
  categories: WeekCategory[]
  types: WeekType[]
  defaultCategoryId?: string | null
  defaultTypeId?: string | null
}

export type WeekItemsParams = {
  page?: number
  categoryId: string
  typeId: string
  endpoint?: string | null
}

export type WeekItemsResult = {
  endpoint: string
  page: number
  total: number
  items: FeedComic[]
}

export async function getHomeFeed(endpoint: string | null = null): Promise<HomeFeedResult> {
  return tauriInvoke<HomeFeedResult>('get_home_feed', { endpoint })
}

export async function getWeekFilters(endpoint: string | null = null): Promise<WeekFiltersResult> {
  return tauriInvoke<WeekFiltersResult>('get_week_filters', { endpoint })
}

export async function getWeekItems({
  page = 1,
  categoryId,
  typeId,
  endpoint = null
}: WeekItemsParams): Promise<WeekItemsResult> {
  return tauriInvoke<WeekItemsResult>('get_week_items', {
    page,
    categoryId,
    typeId,
    endpoint
  })
}

export async function getHomeSectionList({
  mode,
  page = 1,
  sectionId = null,
  sectionTitle = null,
  slug = null,
  type = null,
  filterValue = null,
  category = null,
  week = null,
  order = null,
  endpoint = null
}: HomeSectionListParams): Promise<HomeSectionListResult> {
  return tauriInvoke<HomeSectionListResult>('get_home_section_list', {
    mode,
    page,
    sectionId,
    sectionTitle,
    slug,
    sectionType: type,
    filterValue,
    category,
    week,
    order,
    endpoint
  })
}
