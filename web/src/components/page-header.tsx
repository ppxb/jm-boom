import { ReactNode } from 'react'

type PageHeaderProps = {
  title: string
  description: string
  children?: ReactNode
}

export function PageHeader({ title, description, children }: PageHeaderProps) {
  return (
    <header className="flex flex-col items-start justify-between gap-4 sm:flex-row">
      <div className="flex flex-col gap-1">
        <h1 className="text-4xl font-bold">{title}</h1>
        <p className="mt-2 text-muted-foreground">{description}</p>
      </div>
      {children ? (
        <div className="flex max-w-full flex-wrap items-center gap-2">{children}</div>
      ) : null}
    </header>
  )
}
