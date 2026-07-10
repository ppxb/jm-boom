import { apiClient } from './client'

export type ApiEndpointProbe = {
  endpoint: string
  available: boolean
  latencyMs: number | null
  error: string | null
}

export type EndpointMode = 'auto' | 'manual'

export type EndpointState = {
  mode: EndpointMode
  currentEndpoint: string
  selectedEndpoint: string | null
  endpoints: ApiEndpointProbe[]
}

export type ServerCacheStats = {
  sizeBytes: number
  entryCount: number
}

export type SystemInfo = {
  serverVersion: string
  cache: ServerCacheStats
}

export function getEndpointState(): Promise<EndpointState> {
  return apiClient.get('/api/settings/endpoints')
}

export function refreshApiEndpoints(): Promise<EndpointState> {
  return apiClient.post('/api/settings/endpoints/probe')
}

export function setApiEndpoint(endpoint: string | null): Promise<EndpointState> {
  return apiClient.put('/api/settings/endpoints', { endpoint })
}

export function getSystemInfo(): Promise<SystemInfo> {
  return apiClient.get('/api/settings/system')
}

export function clearServerCache(): Promise<SystemInfo> {
  return apiClient.delete('/api/settings/cache')
}
