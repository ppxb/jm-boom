import { useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import {
  getInstalledSources,
  getSourceCatalog,
  installSource,
  type InstalledSource
} from '@/lib/api/source'
import { queryKeys } from '@/lib/query-keys'
import { useSourceStore } from '@/stores/source-store'

export function useSourceCatalog({ includeCatalog = true }: { includeCatalog?: boolean } = {}) {
  const selectedSourceId = useSourceStore(state => state.selectedSourceId)
  const setSelectedSourceId = useSourceStore(state => state.setSelectedSourceId)
  const resetSelection = useSourceStore(state => state.reset)
  const queryClient = useQueryClient()
  const sourcesQuery = useQuery({
    queryKey: queryKeys.installedSources(),
    queryFn: getInstalledSources,
    staleTime: 30 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })
  const catalogQuery = useQuery({
    queryKey: queryKeys.sourceCatalog(),
    queryFn: () => getSourceCatalog(),
    enabled: includeCatalog,
    staleTime: 5 * 60 * 1000,
    retry: false,
    refetchOnWindowFocus: false
  })
  const installMutation = useMutation({
    mutationFn: installSource,
    onSuccess: installed => {
      queryClient.setQueryData<InstalledSource[]>(queryKeys.installedSources(), current => [
        ...(current ?? []).filter(source => source.info.id !== installed.info.id),
        installed
      ])
      queryClient.invalidateQueries({ queryKey: queryKeys.installedSources() })
      queryClient.invalidateQueries({ queryKey: queryKeys.sourceCatalog() })
      setSelectedSourceId(installed.info.id)
      toast.success(`已安装 ${installed.info.name}`)
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
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
    catalog: catalogQuery.data ?? [],
    selectedSourceId,
    selectedSource: sourcesQuery.data?.find(source => source.info.id === selectedSourceId),
    selectSource: setSelectedSourceId,
    resetSelection,
    installSource: installMutation.mutate,
    installingSourceId: installMutation.isPending ? (installMutation.variables ?? null) : null,
    isLoading: sourcesQuery.isLoading,
    isCatalogLoading: catalogQuery.isLoading,
    catalogError: catalogQuery.error,
    error: sourcesQuery.error
  }
}
