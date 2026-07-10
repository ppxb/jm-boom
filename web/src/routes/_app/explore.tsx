import { createFileRoute, Link, Outlet, useNavigate, useRouterState } from '@tanstack/react-router'
import { SearchIcon } from 'lucide-react'

import { AppPage } from '@/components/app-page'
import { Button } from '@/components/ui/button'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'

export const Route = createFileRoute('/_app/explore')({
  component: ExploreLayout
})

const EXPLORE_TABS = [
  { label: '推荐', value: 'recommend', to: '/explore' },
  { label: '每周', value: 'weekly', to: '/explore/weekly' },
  { label: '排行', value: 'ranking', to: '/explore/ranking' }
] as const

function ExploreLayout() {
  const navigate = useNavigate()
  const pathname = useRouterState({ select: state => state.location.pathname })
  const isSearchPage = pathname.startsWith('/explore/search')
  const activeTab = pathname.startsWith('/explore/weekly')
    ? 'weekly'
    : pathname.startsWith('/explore/ranking')
      ? 'ranking'
      : 'recommend'

  function changeTab(value: string) {
    const tab = EXPLORE_TABS.find(item => item.value === value)

    if (tab) {
      void navigate({ to: tab.to })
    }
  }

  return (
    <AppPage>
      <header className="space-y-4">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <h1 className="text-4xl font-bold">探索</h1>
            <p className="mt-2 text-muted-foreground">发现值得阅读的作品</p>
          </div>

          {!isSearchPage ? (
            <Button asChild variant="outline" size="icon-lg">
              <Link to="/explore/search">
                <SearchIcon className="size-5" />
              </Link>
            </Button>
          ) : null}
        </div>

        {!isSearchPage ? (
          <Tabs value={activeTab} onValueChange={changeTab}>
            <TabsList className="grid w-full grid-cols-3">
              {EXPLORE_TABS.map(tab => (
                <TabsTrigger key={tab.value} value={tab.value}>
                  {tab.label}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        ) : null}
      </header>

      <Outlet />
    </AppPage>
  )
}
