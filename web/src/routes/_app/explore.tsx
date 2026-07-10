import { createFileRoute, Link, Outlet, useNavigate, useRouterState } from '@tanstack/react-router'
import { SearchIcon } from 'lucide-react'

import { AppPage } from '@/components/app-page'
import { PageHeader } from '@/components/page-header'
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
  const isSectionListPage = pathname.startsWith('/explore/list')
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
      {!isSectionListPage ? (
        <PageHeader title="探索" description="发现值得阅读的作品" inlineActions>
          {!isSearchPage ? (
            <Button asChild variant="outline" size="icon-lg">
              <Link to="/explore/search" aria-label="搜索漫画">
                <SearchIcon className="size-5" />
              </Link>
            </Button>
          ) : null}
        </PageHeader>
      ) : null}

      {!isSearchPage && !isSectionListPage ? (
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

      <Outlet />
    </AppPage>
  )
}
