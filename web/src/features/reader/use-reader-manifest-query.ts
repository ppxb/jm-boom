import { useQuery } from '@tanstack/react-query'

import { getComicReadManifest } from '@/lib/api/reader'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'

export function useReaderManifestQuery(comicId: string) {
  return useQuery({
    queryKey: queryKeys.readerManifest(comicId),
    queryFn: () => getComicReadManifest({ readId: comicId }),
    staleTime: CACHE.READER_STALE_TIME,
    gcTime: CACHE.READER_GC_TIME,
    retry: false,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
}
