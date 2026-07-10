import type { ReactNode } from 'react'

export function HomeFeedRail({ children }: { children: ReactNode }) {
  return (
    <div className="-mx-4 flex snap-x snap-mandatory scroll-px-4 scroll-fade-x gap-3 overflow-x-auto px-4 pt-1 pb-4 scrollbar-none sm:mx-0 sm:grid sm:snap-none sm:scroll-px-0 sm:scroll-fade-none sm:grid-cols-3 sm:gap-4 sm:overflow-visible sm:px-0 sm:py-0 lg:grid-cols-4 lg:gap-6">
      {children}
    </div>
  )
}

export function HomeFeedRailItem({ children }: { children: ReactNode }) {
  return (
    <div className="w-[42vw] min-w-36 max-w-48 shrink-0 snap-start sm:w-auto sm:min-w-0 sm:max-w-none">
      {children}
    </div>
  )
}
