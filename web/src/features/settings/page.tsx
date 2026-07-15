import { RotateCcwIcon } from 'lucide-react'
import { useTheme } from 'next-themes'
import { toast } from 'sonner'

import { AppPage } from '@/components/app-page'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { useSourceCatalog } from '@/features/source/use-source-catalog'
import { useSettingsStore } from '@/stores/settings-store'
import { ApiEndpointSection } from './api-endpoint-section'
import { AppearanceSection } from './appearance-section'
import { PrivacySection } from './privacy-section'
import { SourceSection } from './source-section'
import { CacheSection, VersionSection } from './system-sections'
import { useSettingsData } from './use-settings-data'

export function SettingsPage() {
  const { theme = 'system', setTheme } = useTheme()
  const hideCovers = useSettingsStore(state => state.hideCovers)
  const setHideCovers = useSettingsStore(state => state.setHideCovers)
  const reset = useSettingsStore(state => state.reset)
  const {
    sources,
    catalog,
    installSource,
    installingSourceId,
    isLoading,
    isCatalogLoading,
    catalogError
  } = useSourceCatalog()
  const { endpointState, systemInfo, refreshEndpoints, changeEndpoint, clearCache } =
    useSettingsData()

  function resetSettings() {
    reset()
    setTheme('system')
    toast.success('设置已恢复默认')
  }

  return (
    <AppPage contentClassName="max-w-5xl gap-8" showBackTop={false}>
      <PageHeader title="设置" description="APP 设置及缓存管理" inlineActions>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={resetSettings}
          className="text-xs"
        >
          <RotateCcwIcon className="size-4" />
          恢复默认
        </Button>
      </PageHeader>

      <Card>
        <CardContent className="space-y-8">
          <VersionSection info={systemInfo.data} isLoading={systemInfo.isLoading} />

          <Separator />

          <AppearanceSection theme={theme} onThemeChange={setTheme} />

          <Separator />

          <SourceSection
            sources={sources}
            isLoading={isLoading}
            catalog={catalog}
            isCatalogLoading={isCatalogLoading}
            catalogError={catalogError}
            installingSourceId={installingSourceId}
            onInstall={installSource}
          />

          <Separator />

          <ApiEndpointSection
            state={endpointState.data}
            isLoading={endpointState.isLoading}
            isRefreshing={refreshEndpoints.isPending}
            isChanging={changeEndpoint.isPending}
            onEndpointChange={endpoint => changeEndpoint.mutate(endpoint)}
            onRefresh={() => refreshEndpoints.mutate()}
          />

          <Separator />

          <PrivacySection hideCovers={hideCovers} onHideCoversChange={setHideCovers} />

          <Separator />

          <CacheSection
            info={systemInfo.data}
            isLoading={systemInfo.isLoading}
            isClearing={clearCache.isPending}
            onClear={() => clearCache.mutate()}
          />
        </CardContent>
      </Card>
    </AppPage>
  )
}
