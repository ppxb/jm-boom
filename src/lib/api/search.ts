import { invoke } from '@tauri-apps/api/core'

export type SearchAlbum = {
  id: string
  title: string
  author: string
  description: string
  image: string
  tags: string[]
  href: string
  updated_at?: number | null
  isRedirect?: boolean
}

export type SearchAlbumsParams = {
  query: string
  page?: number
  endpoint?: string | null
}

export type SearchAlbumsResult = {
  query: string
  page: number
  total: number
  endpoint?: string | null
  redirectAid?: string | null
  items: SearchAlbum[]
}

export async function searchAlbums({
  query,
  page = 1,
  endpoint = null
}: SearchAlbumsParams): Promise<SearchAlbumsResult> {
  const normalizedQuery = query.trim()

  if (normalizedQuery.length === 0) {
    return {
      query: normalizedQuery,
      page,
      total: 0,
      endpoint: null,
      redirectAid: null,
      items: []
    }
  }

  if (!('__TAURI_INTERNALS__' in window)) {
    throw new Error('Search needs the Tauri desktop runtime. Start the app with the Tauri command.')
  }

  const result = await withTimeout(
    invoke<Partial<SearchAlbumsResult>>('search_comics', {
      query: normalizedQuery,
      page,
      endpoint
    }),
    15000
  )
  console.debug('search_comics raw result', result)

  return {
    query: result.query ?? normalizedQuery,
    page: result.page ?? page,
    total: result.total ?? 0,
    endpoint: result.endpoint ?? null,
    redirectAid: result.redirectAid ?? null,
    items: result.items ?? []
  }
}

function withTimeout<T>(promise: Promise<T>, timeoutMs: number) {
  return new Promise<T>((resolve, reject) => {
    const timeoutId = window.setTimeout(() => {
      reject(new Error('Search timed out. The current API endpoints may be unreachable.'))
    }, timeoutMs)

    promise.then(
      value => {
        window.clearTimeout(timeoutId)
        resolve(value)
      },
      error => {
        window.clearTimeout(timeoutId)
        reject(error)
      }
    )
  })
}
