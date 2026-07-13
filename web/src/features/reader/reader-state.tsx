import { LoaderCircleIcon } from 'lucide-react'

import { cn } from '@/lib/utils'

export function ReaderLoading({ label, className }: { label: string; className?: string }) {
  return (
    <div
      className={cn(
        'flex flex-col items-center justify-center gap-3 text-neutral-400',
        className
      )}
    >
      <LoaderCircleIcon className="size-6 animate-spin" />
      <span className="text-xs">{label}</span>
    </div>
  )
}

export function ReaderError({ title, description }: { title: string; description: string }) {
  return (
    <div className="max-w-md space-y-2 text-center">
      <h1 className="text-lg font-semibold">{title}</h1>
      <p className="text-sm text-neutral-400">{description}</p>
    </div>
  )
}
