import type { ReactNode } from 'react'

import { cn } from '@/lib/utils'

export function SettingsSection({
  icon,
  title,
  children
}: {
  icon: ReactNode
  title: string
  children: ReactNode
}) {
  return (
    <section className="space-y-5">
      <SectionTitle icon={icon} title={title} />
      {children}
    </section>
  )
}

export function SectionTitle({ icon, title }: { icon: ReactNode; title: string }) {
  return (
    <div className="flex items-center gap-2 text-sm font-semibold">
      <span className="text-muted-foreground">{icon}</span>
      {title}
    </div>
  )
}

export function SettingRow({
  title,
  description,
  children,
  inline = false
}: {
  title: string
  description: string
  children: ReactNode
  inline?: boolean
}) {
  return (
    <div
      className={cn(
        'flex flex-col items-stretch justify-between gap-3 sm:flex-row sm:items-center sm:gap-6',
        inline && 'flex-row items-center gap-4'
      )}
    >
      <div className="min-w-0 space-y-1">
        <div className="text-sm font-medium">{title}</div>
        <div className="text-xs leading-5 text-muted-foreground">{description}</div>
      </div>
      <div className="max-w-full shrink-0">{children}</div>
    </div>
  )
}
