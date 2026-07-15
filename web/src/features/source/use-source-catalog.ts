import { useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'

import { getInstalledSources } from '@/lib/api/source'
import { queryKeys } from '@/lib/query-keys'
import { useSourceStore } from '@/stores/source-store'

export function useSourceCatalog() {
  const selectedSourceId = useSourceStore(state => state.selectedSourceId)
  const setSelectedSourceId = useSourceStore(state => state.setSelectedSourceId)
  const resetSelection = useSourceStore(state => state.reset)
  const sourcesQuery = useQuery({
    queryKey: queryKeys.installedSources(),
    queryFn: getInstalledSources,
    staleTime: 30 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })

  useEffect(() => {
    const sources = sourcesQuery.data
    if (!sources) return

    const nextSourceId = sources.some(source => source.info.id === selectedSourceId)
      ? selectedSourceId
      : (sources[0]?.info.id ?? null)
    if (nextSourceId !== selectedSourceId) {
      setSelectedSourceId(nextSourceId)
    }
  }, [selectedSourceId, setSelectedSourceId, sourcesQuery.data])

  return {
    sources: sourcesQuery.data ?? [],
    selectedSourceId,
    selectedSource: sourcesQuery.data?.find(source => source.info.id === selectedSourceId),
    selectSource: setSelectedSourceId,
    resetSelection,
    isLoading: sourcesQuery.isLoading,
    error: sourcesQuery.error
  }
}
