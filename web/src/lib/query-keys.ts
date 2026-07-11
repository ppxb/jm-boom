export const queryKeys = {
  apiEndpointDiscovery: () => ['jm-api-endpoint-discovery'] as const,
  settingsSystem: () => ['jm-settings-system'] as const,
  comicComments: (comicId: string) => ['jm-comic-comments', comicId] as const,
  comicDetail: (comicId: string) => ['jm-comic-detail', comicId] as const,
  downloadTasks: () => ['jm-download-tasks'] as const,
  downloadedChapters: () => ['jm-downloaded-chapters'] as const,
  homeFeed: () => ['jm-home-feed'] as const,
  homeSectionList: (search: unknown) => ['jm-home-section-list', search] as const,
  ranking: (page: number, category: string, order: string) =>
    ['jm-ranking', page, category, order] as const,
  readerManifest: (comicId: string) => ['jm-reader-manifest', comicId] as const,
  readerPage: (comicId: string, index: number, path: string) =>
    ['jm-reader-page', comicId, index, path] as const,
  search: (keyword: string, page: number, sortBy: number) =>
    ['jm-search', keyword, page, sortBy] as const,
  weekFilters: () => ['week-filters'] as const,
  weekItems: (categoryId: string, typeId: string) => ['jm-week-items', categoryId, typeId] as const
}
