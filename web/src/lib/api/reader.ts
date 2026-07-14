import { apiClient, resolveApiUrl } from './client'

export type ComicReadManifestPage = {
  index: number
  name: string
  path: string
}

export type ComicReadManifestResult = {
  readId: string
  pageCount: number
  pages: ComicReadManifestPage[]
}

export async function getComicReadManifest({
  readId
}: {
  readId: string
}): Promise<ComicReadManifestResult> {
  const result = await apiClient.get<{
    chapter_id: string
    pages: Array<{
      index: number
      name: string
      url: string
    }>
  }>(`/api/reader/${readId}/manifest`)

  return {
    readId: result.chapter_id,
    pageCount: result.pages.length,
    pages: result.pages.map(page => ({
      index: page.index,
      name: page.name,
      path: resolveApiUrl(page.url)
    }))
  }
}

export async function preloadComicReadPage(path: string, signal?: AbortSignal) {
  if (typeof Image === 'undefined') {
    return
  }

  const response = await fetch(path, {
    signal,
    headers: {
      'x-jm-boom-image-priority': 'prefetch'
    }
  })
  if (!response.ok) {
    throw new Error(`Failed to preload reader image: HTTP ${response.status}`)
  }

  const objectUrl = URL.createObjectURL(await response.blob())
  const image = new Image()
  image.decoding = 'async'

  try {
    if (typeof image.decode === 'function') {
      image.src = objectUrl
      await image.decode()
      return
    }

    await new Promise<void>((resolve, reject) => {
      image.onload = () => resolve()
      image.onerror = () => reject(new Error(`Failed to decode reader image: ${path}`))
      image.src = objectUrl
    })
  } finally {
    URL.revokeObjectURL(objectUrl)
  }
}
