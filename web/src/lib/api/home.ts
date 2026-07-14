import { apiClient } from './client'
import { mapComicSummary, type ComicSummaryResponse } from './comic-summary'
import type { ComicSummary } from '@/domain/comic'

export type HomeFeedSection = {
  id: string
  title: string
  filterValue: string
  listMode: HomeSectionListMode | null
  rankTag: string
  items: ComicSummary[]
}

export type HomeSectionListMode = 'promote' | 'weekly' | 'latest' | 'ranking'

export type HomeSectionListParams = {
  mode: HomeSectionListMode
  page?: number
  sectionId?: string | null
  sectionTitle?: string | null
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
  items: ComicSummary[]
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
  items: ComicSummary[]
}

type HomeFeedResponse = {
  sections: Array<Omit<HomeFeedSection, 'items'> & { items: ComicSummaryResponse[] }>
}

type HomeSectionListResponse = Omit<HomeSectionListResult, 'items'> & {
  items: ComicSummaryResponse[]
}

type WeekItemsResponse = Omit<WeekItemsResult, 'items'> & {
  items: ComicSummaryResponse[]
}

export async function getHomeFeed(): Promise<HomeFeedResult> {
  const response = await apiClient.get<HomeFeedResponse>('/api/home/feed')
  return {
    sections: response.sections.map(section => ({
      ...section,
      items: section.items.map(mapComicSummary)
    }))
  }
}

export async function getWeekFilters(): Promise<WeekFiltersResult> {
  return apiClient.get('/api/home/weekly/filters')
}

export async function getWeekItems({
  page = 1,
  categoryId,
  typeId
}: WeekItemsParams): Promise<WeekItemsResult> {
  const response = await apiClient.get<WeekItemsResponse>('/api/home/weekly/items', {
    page,
    categoryId,
    typeId
  })
  return { ...response, items: response.items.map(mapComicSummary) }
}

export async function getHomeSectionList({
  mode,
  page = 1,
  sectionId = null,
  sectionTitle = null,
  filterValue = null,
  category = null,
  week = null,
  order = null
}: HomeSectionListParams): Promise<HomeSectionListResult> {
  const response = await apiClient.get<HomeSectionListResponse>('/api/home/sections', {
    mode,
    page,
    sectionId,
    sectionTitle,
    filterValue,
    category,
    week,
    order
  })
  return { ...response, items: response.items.map(mapComicSummary) }
}
