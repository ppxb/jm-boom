import type { ReactNode } from 'react'

interface EmptyStateProps {
  emoji: string
  title: string
  actions?: ReactNode
}

export function EmptyState({ emoji, title, actions }: EmptyStateProps) {
  return (
    <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 text-center">
      <p className="text-6xl font-bold">{emoji}</p>
      <p className="text-sm text-muted-foreground">{title}</p>
      {actions ? <div className="pointer-events-auto mt-1">{actions}</div> : null}
    </div>
  )
}
