import type { UseQueryResult } from '@tanstack/react-query'
import { BugIcon, FolderOpenIcon, LoaderCircleIcon } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Switch } from '@/components/ui/switch'
import type { DiagnosticsInfo } from '@/lib/api/setting'
import { SettingRow, SettingsSection } from './shared'

export function DiagnosticsSection({
  diagnosticsInfo,
  isOpeningDiagnosticsDir,
  isSettingDebugLogging,
  onOpenDiagnosticsDir,
  onDebugLoggingChange
}: {
  diagnosticsInfo: UseQueryResult<DiagnosticsInfo, Error>
  isOpeningDiagnosticsDir: boolean
  isSettingDebugLogging: boolean
  onOpenDiagnosticsDir: () => void
  onDebugLoggingChange: (enabled: boolean) => void
}) {
  return (
    <SettingsSection icon={<BugIcon className="size-4" />} title="调试诊断">
      <SettingRow title="运行日志" description="默认记录运行警告和错误，便于反馈问题">
        <div className="flex items-center gap-2">
          <Input
            disabled
            value={diagnosticsLogDirValue(diagnosticsInfo)}
            title={diagnosticsInfo.data?.logDir ?? ''}
          />
          <Button
            type="button"
            variant="outline"
            size="icon"
            disabled={isOpeningDiagnosticsDir}
            onClick={onOpenDiagnosticsDir}
          >
            {isOpeningDiagnosticsDir ? (
              <LoaderCircleIcon className="size-4 animate-spin" />
            ) : (
              <FolderOpenIcon className="size-4" />
            )}
          </Button>
        </div>
      </SettingRow>
      <SettingRow title="性能调试日志" description="记录阅读器缓存、预取和图片处理耗时">
        <Switch
          checked={diagnosticsInfo.data?.debugLoggingEnabled ?? false}
          disabled={diagnosticsInfo.isLoading || isSettingDebugLogging}
          onCheckedChange={onDebugLoggingChange}
        />
      </SettingRow>
    </SettingsSection>
  )
}

function diagnosticsLogDirValue(stats: UseQueryResult<DiagnosticsInfo, Error>) {
  if (stats.isLoading) {
    return '正在读取路径'
  }

  if (stats.isError) {
    return '读取失败'
  }

  return stats.data?.logDir ?? ''
}
