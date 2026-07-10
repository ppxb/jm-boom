import { ShieldIcon } from 'lucide-react'

import { Switch } from '@/components/ui/switch'
import { SettingRow, SettingsSection } from './shared'

export function PrivacySection({
  hideCovers,
  onHideCoversChange
}: {
  hideCovers: boolean
  onHideCoversChange: (hideCovers: boolean) => void
}) {
  return (
    <SettingsSection icon={<ShieldIcon className="size-4" />} title="NSFW 保护">
      <SettingRow title="封面隐私模式" description="控制列表项是否遮挡封面" inline>
        <Switch checked={hideCovers} onCheckedChange={onHideCoversChange} />
      </SettingRow>
    </SettingsSection>
  )
}
