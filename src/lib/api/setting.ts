import { invoke } from '@tauri-apps/api/core'

export type RemoteSettingParams = {
  endpoint?: string | null
}

export type RemoteSetting = {
  endpoint: string
  imgHost: string
}

export async function getRemoteSetting({
  endpoint = null
}: RemoteSettingParams = {}): Promise<RemoteSetting> {
  if (!('__TAURI_INTERNALS__' in window)) {
    throw new Error('Remote setting needs the Tauri desktop runtime.')
  }

  return invoke<RemoteSetting>('get_remote_setting', { endpoint })
}
