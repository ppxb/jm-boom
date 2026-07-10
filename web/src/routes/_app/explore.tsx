import { createFileRoute, Link, Outlet, useRouterState } from '@tanstack/react-router'
import { SearchIcon } from 'lucide-react'

import { AppPage } from '@/components/app-page'
import { buttonVariants } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export const Route = createFileRoute('/_app/explore')({
  component: ExploreLayout
})

const EXPLORE_TABS = [
  { label: '推荐', to: '/explore' },
  { label: '每周', to: '/explore/weekly' },
  { label: '排行', to: '/explore/ranking' }
] as const

function ExploreLayout() {
  const pathname = useRouterState({ select: state => state.location.pathname })
  const isSearchPage = pathname.startsWith('/explore/search')

  return (
    <AppPage>
      <header className="space-y-4">
        <div>
          <h1 className="text-3xl font-bold">探索</h1>
          <p className="mt-1 text-sm text-muted-foreground">发现值得阅读的作品</p>
        </div>

        {!isSearchPage ? (
          <Link
            to="/explore/search"
            search={{ keyword: '', page: 1, sortBy: 1 }}
            className="flex h-11 w-full items-center gap-3 rounded-xl border border-input bg-background px-4 text-sm text-muted-foreground shadow-xs transition-colors hover:bg-muted/50"
          >
            <SearchIcon className="size-4" />
            搜索漫画或 JM 号
          </Link>
        ) : null}

        {!isSearchPage ? (
          <nav className="sticky top-0 z-30 -mx-4 flex gap-2 overflow-x-auto border-b bg-background/95 px-4 py-2 backdrop-blur sm:mx-0 sm:px-0">
            {EXPLORE_TABS.map(tab => {
              const isActive =
                tab.to === '/explore' ? pathname === '/explore' : pathname.startsWith(tab.to)

              return (
                <Link
                  key={tab.to}
                  to={tab.to}
                  className={cn(
                    buttonVariants({ variant: isActive ? 'default' : 'ghost', size: 'sm' }),
                    'min-w-20 rounded-full'
                  )}
                >
                  {tab.label}
                </Link>
              )
            })}
          </nav>
        ) : null}
      </header>

      <Outlet />
    </AppPage>
  )
}
