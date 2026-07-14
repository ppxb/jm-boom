export type ComicSummary = {
  id: string
  title: string
  author: string
  description: string
  image: string
  tags: string[]
}

export type ComicCardSummary = Pick<ComicSummary, 'id' | 'title' | 'image'>

export type RelatedComic = Pick<ComicSummary, 'id' | 'title' | 'author' | 'image'>

export type ComicChapter = {
  id: string
  title: string
  sort: string
}

export type ComicDetail = Omit<ComicSummary, 'author'> & {
  authors: string[]
  actors: string[]
  works: string[]
  totalViews: number
  likes: number
  commentCount: number
  relatedComics: RelatedComic[]
  chapters: ComicChapter[]
}
