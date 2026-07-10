import { apiClient } from './client'

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
}

export type HomeSectionListResult = {
  mode: HomeSectionListMode
  page: number
  pageSize: number
  total: number
  hasMore: boolean
  title: string
  items: FeedComic[]
}

export type HomeFeedResult = {
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
  categories: WeekCategory[]
  types: WeekType[]
  defaultCategoryId?: string | null
  defaultTypeId?: string | null
}

export type WeekItemsParams = {
  page?: number
  categoryId: string
  typeId: string
}

export type WeekItemsResult = {
  page: number
  total: number
  items: FeedComic[]
}

export async function getHomeFeed(): Promise<HomeFeedResult> {
  return apiClient.get('/api/home/feed')
}

export async function getWeekFilters(): Promise<WeekFiltersResult> {
  return apiClient.get('/api/home/weekly/filters')
}

export async function getWeekItems({
  page = 1,
  categoryId,
  typeId
}: WeekItemsParams): Promise<WeekItemsResult> {
  return apiClient.get('/api/home/weekly/items', { page, categoryId, typeId })
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
  order = null
}: HomeSectionListParams): Promise<HomeSectionListResult> {
  return apiClient.get('/api/home/sections', {
    mode,
    page,
    sectionId,
    sectionTitle,
    slug,
    type,
    filterValue,
    category,
    week,
    order
  })
}
