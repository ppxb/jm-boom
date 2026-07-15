export const queryKeys = {
  apiEndpointDiscovery: () => ['jm-api-endpoint-discovery'] as const,
  settingsSystem: () => ['jm-settings-system'] as const,
  installedSources: () => ['installed-sources'] as const,
  sourceCatalog: () => ['source-catalog'] as const,
  comicComments: (comicId: string) => ['jm-comic-comments', comicId] as const,
  comicDetail: (comicId: string) => ['jm-comic-detail', comicId] as const,
  downloadTasks: () => ['jm-download-tasks'] as const,
  downloadedChapters: () => ['jm-downloaded-chapters'] as const,
  homeFeed: () => ['jm-home-feed'] as const,
  homeSectionList: (search: unknown) => ['jm-home-section-list', search] as const,
  ranking: (page: number, category: string, order: string) =>
    ['jm-ranking', page, category, order] as const,
  readerManifest: (comicId: string) => ['jm-reader-manifest', comicId] as const,
  sourceManga: (sourceId: string, mangaKey: string) =>
    ['source-manga', sourceId, mangaKey] as const,
  sourcePages: (sourceId: string, mangaKey: string, chapterKey: string) =>
    ['source-pages', sourceId, mangaKey, chapterKey] as const,
  sourceSearch: (keyword: string, page: number, sourceIds: string[]) =>
    ['source-search', keyword, page, sourceIds] as const,
  weekFilters: () => ['week-filters'] as const,
  weekItems: (categoryId: string, typeId: string) => ['jm-week-items', categoryId, typeId] as const
}
