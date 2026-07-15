import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import {
  getInstalledSources,
  getSourceCatalog,
  installSource,
  type InstalledSource
} from '@/lib/api/source'
import { queryKeys } from '@/lib/query-keys'

export function useSourceCatalog({ includeCatalog = true }: { includeCatalog?: boolean } = {}) {
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
      toast.success(`已安装 ${installed.info.name}`)
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : String(error))
    }
  })

  return {
    sources: sourcesQuery.data ?? [],
    catalog: catalogQuery.data ?? [],
    installSource: installMutation.mutate,
    installingSourceId: installMutation.isPending ? (installMutation.variables ?? null) : null,
    isLoading: sourcesQuery.isLoading,
    isCatalogLoading: catalogQuery.isLoading,
    catalogError: catalogQuery.error,
    error: sourcesQuery.error
  }
}
