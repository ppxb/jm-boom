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
  const sections = await apiClient.get<
    Array<{
      id: string
      title: string
      slug: string
      type: string
      filter_val: string
      content: Array<{
        id: string
        name: string
        author: string
        description: string
        image: string
        tags: string[]
      }>
    }>
  >('/api/home/feed')

  return {
    endpoint: endpoint || '',
    sections: sections.map(section => ({
      id: section.id,
      title: section.title,
      slug: section.slug,
      type: section.type,
      filterValue: section.filter_val,
      listMode: null, // TODO: determine list mode
      rankTag: '',
      items: section.content.map(comic => ({
        id: comic.id,
        title: comic.name,
        author: comic.author,
        description: comic.description,
        image: comic.image,
        tags: comic.tags,
        updatedAt: null
      }))
    }))
  }
}

export async function getWeekFilters(endpoint: string | null = null): Promise<WeekFiltersResult> {
  // TODO: 实现后端每周放送筛选 API
  return {
    endpoint: endpoint || '',
    categories: [],
    types: [],
    defaultCategoryId: null,
    defaultTypeId: null
  }
}

export async function getWeekItems({
  page = 1,
  categoryId: _categoryId,
  typeId: _typeId,
  endpoint = null
}: WeekItemsParams): Promise<WeekItemsResult> {
  // TODO: 实现后端每周放送列表 API
  return {
    endpoint: endpoint || '',
    page,
    total: 0,
    items: []
  }
}

export async function getHomeSectionList({
  mode,
  page = 1,
  sectionId: _sectionId = null,
  sectionTitle = null,
  slug: _slug = null,
  type: _type = null,
  filterValue: _filterValue = null,
  category: _category = null,
  week: _week = null,
  order: _order = null,
  endpoint = null
}: HomeSectionListParams): Promise<HomeSectionListResult> {
  // TODO: 实现后端分类列表 API
  return {
    endpoint: endpoint || '',
    mode,
    page,
    pageSize: 80,
    total: 0,
    hasMore: false,
    title: sectionTitle || '',
    items: []
  }
}
