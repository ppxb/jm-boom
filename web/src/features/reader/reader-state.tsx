import { LoaderCircleIcon } from 'lucide-react'

export function ReaderLoading({ label }: { label: string }) {
  return (
    <div className="flex items-center gap-2 text-sm text-neutral-300">
      <LoaderCircleIcon className="size-4 animate-spin" />
      {label}
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
