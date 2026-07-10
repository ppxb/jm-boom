import type { ReactNode } from 'react'

export function ComicRail({ children }: { children: ReactNode }) {
  return (
    <div className="-mx-4 flex scroll-fade-x snap-x snap-mandatory scroll-px-4 scrollbar-none gap-3 overflow-x-auto px-4 pt-1 pb-4 sm:mx-0 sm:grid sm:snap-none sm:scroll-px-0 sm:grid-cols-3 sm:gap-4 sm:overflow-visible sm:px-0 sm:py-0 sm:scroll-fade-none lg:grid-cols-4 lg:gap-6">
      {children}
    </div>
  )
}

export function ComicRailItem({ children }: { children: ReactNode }) {
  return (
    <div className="w-[42vw] max-w-48 min-w-36 shrink-0 snap-start sm:w-auto sm:max-w-none sm:min-w-0">
      {children}
    </div>
  )
}
