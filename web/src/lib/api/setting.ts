import { apiClient } from './client'

export type ServerCacheStats = {
  sizeBytes: number
  entryCount: number
  maxSizeBytes: number
}

export type SystemInfo = {
  serverVersion: string
  cache: ServerCacheStats
}

export function getSystemInfo(): Promise<SystemInfo> {
  return apiClient.get('/api/settings/system')
}

export function clearServerCache(): Promise<SystemInfo> {
  return apiClient.delete('/api/settings/cache')
}
