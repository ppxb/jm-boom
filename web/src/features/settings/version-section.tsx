import { openUrl } from '@tauri-apps/plugin-opener'
import {
  DownloadIcon,
  InfoIcon,
  LoaderCircleIcon,
  PackageCheckIcon,
  RefreshCwIcon
} from 'lucide-react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import type { AppUpdateCheckResult } from '@/lib/api/setting'
import { hasTauriRuntime } from '@/lib/api/tauri'
import { cn } from '@/lib/utils'
import { SettingRow, SettingsSection } from './shared'

const PROJECT_REPO_URL = 'https://github.com/ppxb/jm-boom'

export function VersionSection({
  currentVersion,
  update,
  isChecking,
  isInstalling,
  onCheck,
  onInstall
}: {
  currentVersion: string
  update: AppUpdateCheckResult | undefined
  isChecking: boolean
  isInstalling: boolean
  onCheck: () => void
  onInstall: () => void
}) {
  return (
    <SettingsSection icon={<PackageCheckIcon className="size-4" />} title="版本与更新">
      <SettingRow title="当前版本" description="检查 GitHub Releases 上的新版本">
        <AppUpdatePanel
          currentVersion={currentVersion}
          update={update}
          isChecking={isChecking}
          isInstalling={isInstalling}
          onCheck={onCheck}
          onInstall={onInstall}
        />
      </SettingRow>
    </SettingsSection>
  )
}

function AppUpdatePanel({
  currentVersion,
  update,
  isChecking,
  isInstalling,
  onCheck,
  onInstall
}: {
  currentVersion: string
  update: AppUpdateCheckResult | undefined
  isChecking: boolean
  isInstalling: boolean
  onCheck: () => void
  onInstall: () => void
}) {
  const hasUpdate = Boolean(update?.available && update.version)
  const openRepository = () => {
    if (hasTauriRuntime()) {
      void openUrl(PROJECT_REPO_URL).catch(error => {
        toast.error(error instanceof Error ? error.message : String(error))
      })
      return
    }

    window.open(PROJECT_REPO_URL, '_blank', 'noopener,noreferrer')
  }

  return (
    <div className="flex items-center gap-2">
      <div className="inline-flex h-9 items-center gap-2 rounded-4xl border border-border bg-muted/40 px-3 whitespace-nowrap">
        <span className="text-sm font-medium tabular-nums">{formatVersion(currentVersion)}</span>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="-ml-1 size-6 text-muted-foreground hover:text-foreground"
          aria-label="打开 GitHub 仓库"
          title="打开 GitHub 仓库"
          onClick={openRepository}
        >
          <GitHubMark className="size-3.5" />
        </Button>
        <span className="h-3 w-px bg-border" />
        <span
          className={cn(
            'inline-flex items-center gap-1 text-xs',
            hasUpdate ? 'text-primary' : 'text-muted-foreground'
          )}
        >
          {hasUpdate ? (
            <>
              <DownloadIcon className="size-3" />
              可更新至 {formatVersion(update?.version ?? '')}
            </>
          ) : update ? (
            <>
              <PackageCheckIcon className="size-3" />
              已是最新
            </>
          ) : (
            <>
              <InfoIcon className="size-3" />
              未检查
            </>
          )}
        </span>
      </div>
      <Button
        type="button"
        variant={hasUpdate ? 'default' : 'outline'}
        size="sm"
        disabled={isChecking || isInstalling}
        onClick={hasUpdate ? onInstall : onCheck}
      >
        {isInstalling ? (
          <LoaderCircleIcon className="size-4 animate-spin" />
        ) : isChecking ? (
          <RefreshCwIcon className="size-4 animate-spin" />
        ) : hasUpdate ? (
          <DownloadIcon className="size-4" />
        ) : (
          <RefreshCwIcon className="size-4" />
        )}
        {isInstalling ? '正在更新' : isChecking ? '检查中' : hasUpdate ? '立即更新' : '检查更新'}
      </Button>
    </div>
  )
}

function formatVersion(version: string) {
  if (!version || version === '读取中') {
    return version
  }

  return version.startsWith('v') ? version : `v${version}`
}

function GitHubMark({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 24 24" aria-hidden="true" fill="currentColor">
      <path d="M12 .5C5.65.5.5 5.65.5 12c0 5.09 3.29 9.4 7.86 10.93.58.11.79-.25.79-.56v-2.14c-3.2.7-3.87-1.38-3.87-1.38-.52-1.34-1.28-1.7-1.28-1.7-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.56-.29-5.25-1.28-5.25-5.7 0-1.26.45-2.29 1.19-3.09-.12-.29-.52-1.46.11-3.05 0 0 .97-.31 3.16 1.18.92-.26 1.9-.38 2.88-.39.98.01 1.96.13 2.88.39 2.19-1.49 3.16-1.18 3.16-1.18.63 1.59.23 2.76.11 3.05.74.8 1.19 1.83 1.19 3.09 0 4.43-2.7 5.41-5.27 5.7.41.36.78 1.06.78 2.14v3.18c0 .31.21.67.8.56A11.52 11.52 0 0 0 23.5 12C23.5 5.65 18.35.5 12 .5Z" />
    </svg>
  )
}
