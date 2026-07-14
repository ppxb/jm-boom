export type RelatedComic = {
  id: string
  title: string
  author: string
  image: string
}

export type ComicChapter = {
  id: string
  title: string
  sort: string
}

export type ComicDetail = {
  id: string
  title: string
  description: string
  image: string
  authors: string[]
  tags: string[]
  actors: string[]
  works: string[]
  totalViews: number
  likes: number
  commentCount: number
  relatedComics: RelatedComic[]
  chapters: ComicChapter[]
}
