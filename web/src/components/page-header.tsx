import type { ReactNode } from 'react'

import { cn } from '@/lib/utils'

type PageHeaderProps = {
  title: string
  description: string
  children?: ReactNode
  inlineActions?: boolean
}

export function PageHeader({
  title,
  description,
  children,
  inlineActions = false
}: PageHeaderProps) {
  return (
    <header
      className={cn(
        'flex flex-col items-start justify-between gap-4 sm:flex-row',
        inlineActions ? 'flex-row' : null
      )}
    >
      <div className="flex min-w-0 flex-col gap-1">
        <h1 className="text-4xl font-bold">{title}</h1>
        <p className="mt-2 text-muted-foreground">{description}</p>
      </div>
      {children ? (
        <div className="flex max-w-full flex-wrap items-center gap-2">{children}</div>
      ) : null}
    </header>
  )
}
