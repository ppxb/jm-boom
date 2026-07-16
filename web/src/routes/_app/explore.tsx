import { createFileRoute, Link, Outlet, useRouterState } from '@tanstack/react-router'
import { SearchIcon } from 'lucide-react'

import { AppPage } from '@/components/app-page'
import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'

export const Route = createFileRoute('/_app/explore')({
  component: ExploreLayout
})

function ExploreLayout() {
  const pathname = useRouterState({ select: state => state.location.pathname })
  const isSearchPage = pathname.startsWith('/explore/search')
  const isSectionListPage = pathname.startsWith('/explore/list')

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

      <Outlet />
    </AppPage>
  )
}
