export const queryKeys = {
  settingsSystem: () => ['jm-settings-system'] as const,
  installedSources: () => ['installed-sources'] as const,
  sourceCatalog: () => ['source-catalog'] as const,
  comicComments: (comicId: string) => ['jm-comic-comments', comicId] as const,
  comicDetail: (comicId: string) => ['jm-comic-detail', comicId] as const,
  downloadTasks: () => ['jm-download-tasks'] as const,
  downloadedChapters: () => ['jm-downloaded-chapters'] as const,
  readerManifest: (comicId: string) => ['jm-reader-manifest', comicId] as const,
  sourceManga: (sourceId: string, mangaKey: string) =>
    ['source-manga', sourceId, mangaKey] as const,
  sourceListing: (sourceId: string, listingId: string, page: number) =>
    ['source-listing', sourceId, listingId, page] as const,
  sourceListingPreviews: (sourceVersions: string[]) =>
    ['source-listing-previews', sourceVersions] as const,
  sourcePages: (sourceId: string, mangaKey: string, chapterKey: string) =>
    ['source-pages', sourceId, mangaKey, chapterKey] as const,
  sourceSearch: (keyword: string, page: number, sourceIds: string[]) =>
    ['source-search', keyword, page, sourceIds] as const,
}
