import type { ReactNode } from 'react'

import { cn } from '@/lib/utils'

interface EmptyStateProps {
  emoji: string
  title: string
  actions?: ReactNode
  className?: string
}

export function EmptyState({ emoji, title, actions, className }: EmptyStateProps) {
  return (
    <div
      className={cn(
        'flex min-h-80 w-full flex-col items-center justify-center gap-3 text-center',
        className
      )}
    >
      <p className="text-6xl font-bold">{emoji}</p>
      <p className="text-sm text-muted-foreground">{title}</p>
      {actions ? <div className="pointer-events-auto mt-1">{actions}</div> : null}
    </div>
  )
}
