import type { ReactNode } from 'react'

import { BackTopButton } from '@/components/back-top-button'
import { cn } from '@/lib/utils'

type AppPageProps = {
  children: ReactNode
  className?: string
  contentClassName?: string
  showBackTop?: boolean
}

export function AppPage({
  children,
  className,
  contentClassName,
  showBackTop = true
}: AppPageProps) {
  return (
    <main
      className={cn('relative flex min-h-dvh flex-col bg-background px-4 pt-6 pb-24', className)}
    >
      <div className={cn('mx-auto flex w-full max-w-6xl flex-1 flex-col gap-6', contentClassName)}>
        {children}
      </div>
      {showBackTop ? <BackTopButton /> : null}
    </main>
  )
}
