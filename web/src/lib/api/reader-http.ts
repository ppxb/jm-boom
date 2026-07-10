import { apiClient } from './client'

// ============== Types ==============

export type PageInfo = {
  index: number
  name: string
  url: string
}

export type ManifestResponse = {
  chapter_id: string
  pages: PageInfo[]
}

// ============== API Functions ==============

const DEFAULT_ENDPOINT = 'https://www.cdnhjk.net'

/**
 * 获取章节图片清单
 */
export async function getChapterManifest(
  chapterId: string,
  endpoint: string = DEFAULT_ENDPOINT
): Promise<ManifestResponse> {
  return apiClient.get<ManifestResponse>(
    `/api/reader/${chapterId}/manifest`,
    { endpoint }
  )
}

/**
 * 获取单页图片 URL（已解扰 + WebP）
 */
export function getPageImageUrl(
  chapterId: string,
  pageIndex: number,
  endpoint: string = DEFAULT_ENDPOINT
): string {
  const params = new URLSearchParams({ endpoint })
  return `/api/reader/${chapterId}/pages/${pageIndex}?${params}`
}

/**
 * 下载单页图片（Blob）
 */
export async function downloadPageImage(
  chapterId: string,
  pageIndex: number,
  endpoint: string = DEFAULT_ENDPOINT
): Promise<Blob> {
  return apiClient.get<Blob>(
    `/api/reader/${chapterId}/pages/${pageIndex}`,
    { endpoint }
  )
}
