import { apiClient } from './client'

export type SourceInfo = {
  id: string
  name: string
  version: number
  altNames: string[]
  url: string | null
  urls: string[]
  languages: string[]
  contentRating: number | null
  minAppVersion: string | null
  maxAppVersion: string | null
}

export type SourceCapabilities = {
  providesHome: boolean
  providesListings: boolean
  dynamicListings: boolean
  dynamicFilters: boolean
  dynamicSettings: boolean
  providesImageRequests: boolean
  processesPages: boolean
  providesPageDescriptions: boolean
  providesAlternateCovers: boolean
  providesBaseUrl: boolean
  handlesNotifications: boolean
  handlesDeepLinks: boolean
  handlesBasicLogin: boolean
  handlesWebLogin: boolean
  handlesMigration: boolean
  sendsPartialResults: boolean
  usesNetwork: boolean
  usesHtml: boolean
  usesCanvas: boolean
  usesDefaults: boolean
  usesJavascript: boolean
}

export type InstalledSource = {
  info: SourceInfo
  capabilities: SourceCapabilities
  filterCount: number
  settingCount: number
}

export function getInstalledSources() {
  return apiClient.get<InstalledSource[]>('/api/sources')
}
