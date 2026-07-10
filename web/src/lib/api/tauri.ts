import { invoke } from '@tauri-apps/api/core'

type TauriInvokeArgs = Record<string, unknown> | number[] | ArrayBuffer | Uint8Array

const DEFAULT_TAURI_RUNTIME_MESSAGE = 'This content needs the Tauri desktop runtime.'

type TauriCommandErrorPayload = {
  kind: string
  message: string
  retryable: boolean
}

export class TauriCommandError extends Error {
  kind: string
  retryable: boolean
  raw: unknown

  constructor(payload: TauriCommandErrorPayload, raw: unknown) {
    super(payload.message)
    this.name = 'TauriCommandError'
    this.kind = payload.kind
    this.retryable = payload.retryable
    this.raw = raw
  }
}

export function hasTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

export function ensureTauriRuntime(message = DEFAULT_TAURI_RUNTIME_MESSAGE) {
  if (!hasTauriRuntime()) {
    throw new Error(message)
  }
}

export function tauriInvoke<T>(
  command: string,
  args?: TauriInvokeArgs,
  runtimeMessage = DEFAULT_TAURI_RUNTIME_MESSAGE
): Promise<T> {
  ensureTauriRuntime(runtimeMessage)

  return invoke<T>(command, args).catch((error: unknown) => {
    throw normalizeTauriInvokeError(error)
  })
}

function normalizeTauriInvokeError(error: unknown): Error {
  if (isTauriCommandErrorPayload(error)) {
    return new TauriCommandError(error, error)
  }

  if (error instanceof Error) {
    return error
  }

  return new Error(typeof error === 'string' ? error : JSON.stringify(error))
}

function isTauriCommandErrorPayload(error: unknown): error is TauriCommandErrorPayload {
  return (
    typeof error === 'object' &&
    error !== null &&
    'kind' in error &&
    'message' in error &&
    'retryable' in error &&
    typeof error.kind === 'string' &&
    typeof error.message === 'string' &&
    typeof error.retryable === 'boolean'
  )
}
