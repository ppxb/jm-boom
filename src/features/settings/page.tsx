import { RotateCcwIcon } from 'lucide-react'
import { useTheme } from 'next-themes'
import { toast } from 'sonner'

import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { useSettingsStore } from '@/stores/settings-store'
import { AccountSection } from './account-section'
import { ApiEndpointSection } from './api-endpoint-section'
import { AppearanceSection } from './appearance-section'
import { CacheSection } from './cache-section'
import { DiagnosticsSection } from './diagnostics-section'
import { PrivacySection } from './privacy-section'
import { ProxySection } from './proxy-section'
import { VersionSection } from './version-section'
import { useEndpointOptions } from './use-endpoint-options'
import { useSettingsData } from './use-settings-data'
import { useAutoEndpointSelection } from './use-auto-endpoint'

export function SettingsPage() {
  const { theme = 'system', setTheme } = useTheme()
  const api = useSettingsStore(state => state.api)
  const readerCacheLimitMb = useSettingsStore(state => state.readerCacheLimitMb)
  const cacheLimitBytes = readerCacheLimitMb * 1024 * 1024
  const proxyMode = useSettingsStore(state => state.proxyMode)
  const proxyHost = useSettingsStore(state => state.proxyHost)
  const proxyPort = useSettingsStore(state => state.proxyPort)
  const hideCovers = useSettingsStore(state => state.hideCovers)
  const setApi = useSettingsStore(state => state.setApi)
  const setReaderCacheLimitMb = useSettingsStore(state => state.setReaderCacheLimitMb)
  const setProxyMode = useSettingsStore(state => state.setProxyMode)
  const setProxyHost = useSettingsStore(state => state.setProxyHost)
  const setProxyPort = useSettingsStore(state => state.setProxyPort)
  const setHideCovers = useSettingsStore(state => state.setHideCovers)
  const reset = useSettingsStore(state => state.reset)

  const {
    endpointDiscovery,
    readerCacheStats,
    clearCache,
    openCacheDir,
    savedLoginConfig,
    saveAccount,
    setAccountAutoLogin,
    diagnosticsInfo,
    openDiagnosticsDir,
    setDiagnosticsDebug,
    appVersion,
    appUpdate,
    checkUpdate,
    installUpdate
  } = useSettingsData(api, cacheLimitBytes, proxyMode, proxyHost, proxyPort)

  const endpointOptions = useEndpointOptions(api, endpointDiscovery.data)
  const { isRefreshingEndpoints, setIsRefreshingEndpoints } = useAutoEndpointSelection(
    endpointDiscovery.data,
    endpointDiscovery.dataUpdatedAt
  )

  function refreshEndpoints() {
    setIsRefreshingEndpoints(true)
    void endpointDiscovery.refetch().catch(() => {
      setIsRefreshingEndpoints(false)
    })
  }

  function resetSettings() {
    reset()
    setTheme('system')
    toast.success('设置已恢复默认')
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-5xl space-y-8 p-[32px_32px_16px_96px]">
        <PageHeader title="设置" description="调整 APP 配置和内容显示偏好">
          <Button variant="outline" size="sm" onClick={resetSettings} className="text-xs">
            <RotateCcwIcon className="size-4" />
            恢复默认
          </Button>
        </PageHeader>

        <Card>
          <CardContent className="space-y-8">
            <VersionSection
              currentVersion={
                checkUpdate.data?.currentVersion ||
                appUpdate.data?.currentVersion ||
                appVersion.data ||
                '读取中'
              }
              update={checkUpdate.data ?? appUpdate.data}
              isChecking={checkUpdate.isPending}
              isInstalling={installUpdate.isPending}
              onCheck={() => checkUpdate.mutate()}
              onInstall={() => installUpdate.mutate()}
            />

            <Separator />

            <AppearanceSection theme={theme} onThemeChange={setTheme} />

            <Separator />

            <ApiEndpointSection
              endpoint={api}
              endpointOptions={endpointOptions}
              isDiscovering={endpointDiscovery.isFetching}
              isRefreshingEndpoints={isRefreshingEndpoints}
              onEndpointChange={setApi}
              onRefresh={refreshEndpoints}
            />

            <Separator />

            <ProxySection
              proxyMode={proxyMode}
              proxyHost={proxyHost}
              proxyPort={proxyPort}
              onProxyModeChange={setProxyMode}
              onProxyHostChange={setProxyHost}
              onProxyPortChange={setProxyPort}
            />

            <Separator />

            <CacheSection
              readerCacheLimitMb={readerCacheLimitMb}
              stats={readerCacheStats}
              isOpeningCacheDir={openCacheDir.isPending}
              isClearingCache={clearCache.isPending}
              onCacheLimitChange={setReaderCacheLimitMb}
              onOpenCacheDir={() => openCacheDir.mutate()}
              onClearCache={() => clearCache.mutate()}
            />

            <Separator />

            <PrivacySection hideCovers={hideCovers} onHideCoversChange={setHideCovers} />

            <Separator />

            <AccountSection
              savedLoginConfig={savedLoginConfig.data}
              isLoading={savedLoginConfig.isLoading}
              isSaving={saveAccount.isPending}
              isSettingAutoLogin={setAccountAutoLogin.isPending}
              onAutoLoginChange={autoLogin => setAccountAutoLogin.mutate(autoLogin)}
              onCredentialsChange={input => saveAccount.mutate(input)}
            />

            <Separator />

            <DiagnosticsSection
              diagnosticsInfo={diagnosticsInfo}
              isOpeningDiagnosticsDir={openDiagnosticsDir.isPending}
              isSettingDebugLogging={setDiagnosticsDebug.isPending}
              onOpenDiagnosticsDir={() => openDiagnosticsDir.mutate()}
              onDebugLoggingChange={enabled => setDiagnosticsDebug.mutate(enabled)}
            />
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
