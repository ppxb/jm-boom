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
  maxSizeBytes: number
}

export type SystemInfo = {
  serverVersion: string
  cache: ServerCacheStats
}

export type AccountLoginStatus = 'loggedOut' | 'loggingIn' | 'loggedIn'
export type AccountSignInStatus = 'pending' | 'signingIn' | 'signedIn'

export type AccountState = {
  username: string | null
  autoLogin: boolean
  autoSignIn: boolean
  loginStatus: AccountLoginStatus
  signInStatus: AccountSignInStatus
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

export function getAccountState(): Promise<AccountState> {
  return apiClient.get('/api/settings/account')
}

export function updateAccount(input: {
  username: string
  password?: string
  autoLogin: boolean
  autoSignIn: boolean
}): Promise<AccountState> {
  return apiClient.put('/api/settings/account', input)
}

export function clearAccount(): Promise<AccountState> {
  return apiClient.delete('/api/settings/account')
}
