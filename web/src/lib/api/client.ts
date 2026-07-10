/**
 * HTTP API Client for JM Boom Server
 *
 * 根据环境自动选择：
 * - 开发环境：连接本地后端 http://localhost:3000
 * - 生产环境：使用相对路径 /api
 */

const API_BASE_URL = import.meta.env.DEV
  ? 'http://localhost:3000'
  : ''

export function resolveApiUrl(path: string) {
  if (/^https?:\/\//i.test(path)) {
    return path
  }

  return `${API_BASE_URL}${path}`
}

export class ApiError extends Error {
  constructor(
    message: string,
    public status?: number,
    public data?: any
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

async function request<T>(
  path: string,
  options?: RequestInit
): Promise<T> {
  const url = resolveApiUrl(path)

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options?.headers,
      },
    })

    if (!response.ok) {
      let errorData: any
      try {
        errorData = await response.json()
      } catch {
        errorData = { error: response.statusText }
      }

      throw new ApiError(
        errorData.error || `HTTP ${response.status}`,
        response.status,
        errorData
      )
    }

    // 如果是图片响应，返回 blob
    if (response.headers.get('content-type')?.startsWith('image/')) {
      return response.blob() as T
    }

    return response.json()
  } catch (error) {
    if (error instanceof ApiError) {
      throw error
    }

    throw new ApiError(
      error instanceof Error ? error.message : 'Network error',
      undefined,
      error
    )
  }
}

export const apiClient = {
  get: <T>(path: string, params?: Record<string, any>) => {
    const query = params
      ? '?' + new URLSearchParams(
          Object.entries(params).reduce((acc, [key, value]) => {
            if (value !== null && value !== undefined) {
              acc[key] = String(value)
            }
            return acc
          }, {} as Record<string, string>)
        ).toString()
      : ''

    return request<T>(`${path}${query}`)
  },

  post: <T>(path: string, data?: any) =>
    request<T>(path, {
      method: 'POST',
      body: JSON.stringify(data),
    }),

  put: <T>(path: string, data?: any) =>
    request<T>(path, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),

  delete: <T>(path: string) =>
    request<T>(path, { method: 'DELETE' }),
}
