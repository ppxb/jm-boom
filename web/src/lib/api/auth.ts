import { apiClient } from './client'

export type AccessGateConfig = {
  enabled: boolean
}

export type AccessGateLoginResult = {
  success: boolean
}

export function getAccessGateConfig(): Promise<AccessGateConfig> {
  return apiClient.get('/api/auth/config')
}

export function loginAccessGate(password: string): Promise<AccessGateLoginResult> {
  return apiClient.post('/api/auth/login', { password })
}
