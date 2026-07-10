import { getVersion } from '@tauri-apps/api/app'
import { hasTauriRuntime, tauriInvoke } from './tauri'

export type RemoteSettingParams = {
  endpoint?: string | null
}

export type RemoteSetting = {
  endpoint: string
  imgHost: string
}

export type ApiEndpointProbe = {
  endpoint: string
  available: boolean
  latencyMs: number | null
  imgHost: string | null
  error: string | null
}

export type NetworkProxyMode = 'off' | 'http' | 'socks5'

export type AppUpdateCheckResult = {
  currentVersion: string
  available: boolean
  version: string | null
  notes: string | null
  pubDate: string | null
}

export type DiagnosticsInfo = {
  logDir: string
  debugLoggingEnabled: boolean
}

export async function getRemoteSetting({
  endpoint = null
}: RemoteSettingParams = {}): Promise<RemoteSetting> {
  return tauriInvoke<RemoteSetting>(
    'get_remote_setting',
    { endpoint },
    'Remote setting needs the Tauri desktop runtime.'
  )
}

export async function discoverApiEndpoints(): Promise<ApiEndpointProbe[]> {
  return tauriInvoke<ApiEndpointProbe[]>(
    'discover_api_endpoints',
    undefined,
    'API endpoint discovery needs the Tauri desktop runtime.'
  )
}

export async function configureNetworkProxy({
  mode,
  host,
  port
}: {
  mode: NetworkProxyMode
  host: string
  port: number
}): Promise<void> {
  if (!hasTauriRuntime()) {
    return
  }

  return tauriInvoke<void>('configure_network_proxy', { mode, host, port })
}

export async function getCurrentAppVersion(): Promise<string> {
  if (!hasTauriRuntime()) {
    return ''
  }

  return getVersion()
}

export async function checkAppUpdate({
  force = false
}: {
  force?: boolean
} = {}): Promise<AppUpdateCheckResult> {
  if (!hasTauriRuntime()) {
    return {
      currentVersion: '',
      available: false,
      version: null,
      notes: null,
      pubDate: null
    }
  }

  return tauriInvoke<AppUpdateCheckResult>('check_app_update', { force })
}

export async function installAppUpdate(): Promise<boolean> {
  if (!hasTauriRuntime()) {
    return false
  }

  return tauriInvoke<boolean>('install_app_update')
}

export async function getDiagnosticsInfo(): Promise<DiagnosticsInfo> {
  if (!hasTauriRuntime()) {
    return emptyDiagnosticsInfo()
  }

  return tauriInvoke<DiagnosticsInfo>('get_diagnostics_info')
}

export async function openDiagnosticsLogDir(): Promise<void> {
  if (!hasTauriRuntime()) {
    return
  }

  return tauriInvoke<void>('open_diagnostics_log_dir')
}

export async function setDiagnosticsDebugLogging(enabled: boolean): Promise<DiagnosticsInfo> {
  if (!hasTauriRuntime()) {
    return {
      ...emptyDiagnosticsInfo(),
      debugLoggingEnabled: enabled
    }
  }

  return tauriInvoke<DiagnosticsInfo>('set_diagnostics_debug_logging', { enabled })
}

function emptyDiagnosticsInfo(): DiagnosticsInfo {
  return {
    logDir: '',
    debugLoggingEnabled: import.meta.env.DEV
  }
}
