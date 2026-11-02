/**
 * HTTP API Client for JM Boom Server
 *
 * API 请求统一使用同源路径：
 * - 开发环境由 Vite 将 /api 代理到本地后端
 * - 生产环境由 Web Server 将 /api 转发到后端
 */

export function resolveApiUrl(path: string) {
  if (/^https?:\/\//i.test(path)) {
    return path
  }

  return `/${path.replace(/^\/+/, '')}`
}

export class ApiError extends Error {
  constructor(
    message: string,
    public status?: number,
    public data?: unknown,
    public retryable = false
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export type DownloadFile = { blob: Blob; filename: string }

async function request<T>(
  path: string,
  options?: RequestInit,
  responseType: 'json' | 'file' = 'json'
): Promise<T> {
  const url = resolveApiUrl(path)

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers
      }
    })

    if (!response.ok) {
      const error = await readErrorResponse(response)
      throw new ApiError(error.message, response.status, error.data, error.retryable)
    }

    if (responseType === 'file') {
      let filename: string
      try {
        filename = downloadFilename(response.headers)
      } catch (error) {
        await response.body?.cancel()
        throw error
      }
      return { blob: await response.blob(), filename } as T
    }

    // 如果是图片响应，返回 blob
    if (response.headers.get('content-type')?.startsWith('image/')) {
      return response.blob() as T
    }

    if (response.status === 204) {
      return undefined as T
    }

    return response.json()
  } catch (error) {
    if (error instanceof ApiError) {
      throw error
    }

    if (isAbortError(error)) {
      throw error
    }

    throw new ApiError(
      error instanceof Error ? error.message : 'Network error',
      undefined,
      error,
      true
    )
  }
}

function downloadFilename(headers: Headers) {
  const disposition = headers.get('content-disposition') ?? ''
  const encoded = /filename\*=UTF-8''([^;]+)/i.exec(disposition)?.[1]
  if (encoded) {
    try {
      const filename = decodeURIComponent(encoded.trim())
      if (filename) return filename
    } catch {
      /* Fall back to an ASCII filename. */
    }
  }
  const filename = /filename="([^"]+)"/i.exec(disposition)?.[1]
  if (!filename) throw new ApiError('下载响应缺少有效文件名')
  return filename
}

async function readErrorResponse(response: Response) {
  const fallbackMessage = response.statusText || `HTTP ${response.status}`
  const rawBody = await response.text()
  let data: unknown = rawBody

  if (rawBody.length > 0) {
    try {
      data = JSON.parse(rawBody)
    } catch {
      return {
        message: rawBody,
        data,
        retryable: isRetryableStatus(response.status)
      }
    }
  }

  if (isRecord(data)) {
    return {
      message:
        typeof data.error === 'string' && data.error.length > 0 ? data.error : fallbackMessage,
      data,
      retryable:
        typeof data.retryable === 'boolean' ? data.retryable : isRetryableStatus(response.status)
    }
  }

  return {
    message: fallbackMessage,
    data,
    retryable: isRetryableStatus(response.status)
  }
}

function isRetryableStatus(status: number) {
  return status === 408 || status === 425 || status === 429 || status >= 500
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isAbortError(error: unknown) {
  return isRecord(error) && error.name === 'AbortError'
}

export const apiClient = {
  getFile: (path: string, options?: RequestInit) => request<DownloadFile>(path, options, 'file'),

  get: <T>(path: string, params?: Record<string, unknown>, options?: RequestInit) => {
    const query = params
      ? '?' +
        new URLSearchParams(
          Object.entries(params).reduce(
            (acc, [key, value]) => {
              if (value !== null && value !== undefined) {
                acc[key] = String(value)
              }
              return acc
            },
            {} as Record<string, string>
          )
        ).toString()
      : ''

    return request<T>(`${path}${query}`, options)
  },

  post: <T>(path: string, data?: unknown) =>
    request<T>(path, {
      method: 'POST',
      body: JSON.stringify(data)
    }),

  put: <T>(path: string, data?: unknown, options?: RequestInit) =>
    request<T>(path, {
      ...options,
      method: 'PUT',
      body: JSON.stringify(data)
    }),

  delete: <T>(path: string) => request<T>(path, { method: 'DELETE' })
}
