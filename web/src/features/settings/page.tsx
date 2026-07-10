import { RotateCcwIcon } from 'lucide-react'
import { useTheme } from 'next-themes'
import { toast } from 'sonner'

import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { useSettingsStore } from '@/stores/settings-store'
import { ApiEndpointSection } from './api-endpoint-section'
import { AppearanceSection } from './appearance-section'
import { PrivacySection } from './privacy-section'
import { useSettingsData } from './use-settings-data'

export function SettingsPage() {
  const { theme = 'system', setTheme } = useTheme()
  const hideCovers = useSettingsStore(state => state.hideCovers)
  const setHideCovers = useSettingsStore(state => state.setHideCovers)
  const reset = useSettingsStore(state => state.reset)
  const { endpointState, refreshEndpoints, changeEndpoint } = useSettingsData()

  function resetSettings() {
    reset()
    setTheme('system')
    toast.success('设置已恢复默认')
  }

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="mx-auto w-full max-w-5xl space-y-8 px-4 pt-6 pb-36 sm:px-6 sm:pb-28 lg:px-8">
        <PageHeader title="设置" description="调整服务接口和内容显示偏好">
          <Button variant="outline" size="sm" onClick={resetSettings} className="text-xs">
            <RotateCcwIcon className="size-4" />
            恢复默认
          </Button>
        </PageHeader>

        <Card>
          <CardContent className="space-y-8">
            <AppearanceSection theme={theme} onThemeChange={setTheme} />

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
          </CardContent>
        </Card>
      </div>
    </main>
  )
}
