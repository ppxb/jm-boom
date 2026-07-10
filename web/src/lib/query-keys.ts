export const queryKeys = {
  apiEndpointDiscovery: () => ['jm-api-endpoint-discovery'] as const,
  comicComments: (comicId: string) => ['jm-comic-comments', comicId] as const,
  comicDetail: (comicId: string) => ['jm-comic-detail', comicId] as const,
  downloadTasks: () => ['jm-download-tasks'] as const,
  favorites: (folderId: string, page: number) => ['jm-favorites', folderId, page] as const,
  homeFeed: () => ['jm-home-feed'] as const,
  homeSectionList: (search: unknown) => ['jm-home-section-list', search] as const,
  ranking: (page: number, category: string, order: string) =>
    ['jm-ranking', page, category, order] as const,
  readerManifest: (comicId: string) => ['jm-reader-manifest', comicId] as const,
  readerPage: (comicId: string, index: number) => ['jm-reader-page', comicId, index] as const,
  search: (keyword: string, page: number, sortBy: number) =>
    ['jm-search', keyword, page, sortBy] as const,
  signInData: (userId: number | undefined) => ['jm-sign-in-data', userId] as const,
  weekFilters: () => ['week-filters'] as const,
  weekItems: (categoryId: string, typeId: string) => ['jm-week-items', categoryId, typeId] as const
}
