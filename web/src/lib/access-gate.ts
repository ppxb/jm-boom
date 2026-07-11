import { getAccessGateConfig } from '@/lib/api/auth'

const ACCESS_GATE_KEY = 'jm-boom-access-granted'

let configRequest: ReturnType<typeof getAccessGateConfig> | undefined

export function loadAccessGateConfig() {
  configRequest ??= getAccessGateConfig().catch(error => {
    configRequest = undefined
    throw error
  })
  return configRequest
}

export function hasAccessGateGrant() {
  return sessionStorage.getItem(ACCESS_GATE_KEY) === 'true'
}

export function grantAccessGate() {
  sessionStorage.setItem(ACCESS_GATE_KEY, 'true')
}

export function clearAccessGateGrant() {
  sessionStorage.removeItem(ACCESS_GATE_KEY)
}
