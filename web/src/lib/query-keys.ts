type QueryEndpoint = string | null | undefined

export const queryKeys = {
  apiEndpointDiscovery: () => ['jm-api-endpoint-discovery'] as const,
  appUpdate: () => ['app-update'] as const,
  appVersion: () => ['app-version'] as const,
  comicComments: (endpoint: QueryEndpoint, comicId: string) =>
    ['jm-comic-comments', endpoint, comicId] as const,
  comicDetail: (endpoint: QueryEndpoint, comicId: string) =>
    ['jm-comic-detail', endpoint, comicId] as const,
  diagnosticsInfo: () => ['diagnostics-info'] as const,
  downloadTasks: () => ['jm-download-tasks'] as const,
  favorites: (endpoint: QueryEndpoint, folderId: string, page: number) =>
    ['jm-favorites', endpoint, folderId, page] as const,
  homeFeed: (endpoint: QueryEndpoint) => ['jm-home-feed', endpoint] as const,
  homeSectionList: (endpoint: QueryEndpoint, search: unknown) =>
    ['jm-home-section-list', endpoint, search] as const,
  savedLoginConfig: () => ['saved-login-config'] as const,
  ranking: (endpoint: QueryEndpoint, page: number, category: string, order: string) =>
    ['jm-ranking', endpoint, page, category, order] as const,
  readerCacheStats: (cacheLimitBytes: number) => ['reader-cache-stats', cacheLimitBytes] as const,
  readerManifest: (endpoint: QueryEndpoint, comicId: string) =>
    ['jm-reader-manifest', endpoint, comicId] as const,
  readerPage: (endpoint: QueryEndpoint, comicId: string, cacheLimitBytes: number, index: number) =>
    ['jm-reader-page', endpoint, comicId, cacheLimitBytes, index] as const,
  search: (endpoint: QueryEndpoint, keyword: string, page: number, sortBy: number) =>
    ['jm-search', endpoint, keyword, page, sortBy] as const,
  signInData: (endpoint: QueryEndpoint, userId: number | undefined) =>
    ['jm-sign-in-data', endpoint, userId] as const,
  weekFilters: (endpoint: QueryEndpoint) => ['week-filters', endpoint] as const,
  weekItems: (endpoint: QueryEndpoint, categoryId: string, typeId: string) =>
    ['jm-week-items', endpoint, categoryId, typeId] as const
}
