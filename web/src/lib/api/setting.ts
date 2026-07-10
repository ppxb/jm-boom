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

export function getEndpointState(): Promise<EndpointState> {
  return apiClient.get('/api/settings/endpoints')
}

export function refreshApiEndpoints(): Promise<EndpointState> {
  return apiClient.post('/api/settings/endpoints/probe')
}

export function setApiEndpoint(endpoint: string | null): Promise<EndpointState> {
  return apiClient.put('/api/settings/endpoints', { endpoint })
}
