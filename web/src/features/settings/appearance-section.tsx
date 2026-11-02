import { MonitorCogIcon, MonitorIcon, MoonIcon, SunIcon } from 'lucide-react'

import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { isTheme, type Theme } from '@/lib/theme'
import { SettingRow, SettingsSection } from './shared'

const THEME_OPTIONS = [
  { value: 'system', icon: MonitorIcon, label: '跟随系统' },
  { value: 'light', icon: SunIcon, label: '浅色' },
  { value: 'dark', icon: MoonIcon, label: '深色' }
]

export function AppearanceSection({
  theme,
  onThemeChange
}: {
  theme: Theme
  onThemeChange: (theme: Theme) => void
}) {
  return (
    <SettingsSection icon={<MonitorCogIcon className="size-4" />} title="外观">
      <SettingRow title="主题" description="控制应用的明暗色主题" inline>
        <Tabs
          value={theme}
          onValueChange={value => {
            if (isTheme(value)) onThemeChange(value)
          }}
        >
          <TabsList>
            {THEME_OPTIONS.map(option => (
              <TabsTrigger key={option.value} value={option.value}>
                <option.icon className="size-4" />
                <span className="sr-only">{option.label}</span>
              </TabsTrigger>
            ))}
          </TabsList>
        </Tabs>
      </SettingRow>
    </SettingsSection>
  )
}
