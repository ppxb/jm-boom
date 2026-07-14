import type { ComicSummary } from '@/domain/comic'

export type ComicSummaryResponse = {
  id: string
  title: string
  author: string
  description: string
  image: string
  tags: string[]
}

export function mapComicSummary(response: ComicSummaryResponse): ComicSummary {
  return {
    id: response.id,
    title: response.title,
    author: response.author,
    description: response.description,
    image: response.image,
    tags: response.tags
  }
}
