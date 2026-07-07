import { useQuery } from '@tanstack/react-query'

import { FeedHeader, StatePanel } from '@/components/comic-feed'
import { getHomeFeed, type HomeFeedSection } from '@/lib/api/home'
import { LIST_QUERY_GC_TIME, LIST_QUERY_STALE_TIME } from '@/lib/query-cache'
import { queryKeys } from '@/lib/query-keys'
import { useSettingsStore } from '@/stores/settings-store'
import { BackTop } from './back-top'
import { HomeFeedDirectory } from './home-directory'
import { HomeFeedSections } from './home-feed-sections'
import { HomeFeedSkeleton } from './home-feed-skeleton'

const EMPTY_HOME_SECTIONS: HomeFeedSection[] = []

export function HomePage() {
  const endpoint = useSettingsStore(state => state.api)
  const homeFeed = useQuery({
    queryKey: queryKeys.homeFeed(endpoint),
    queryFn: () => getHomeFeed(endpoint),
    staleTime: LIST_QUERY_STALE_TIME,
    gcTime: LIST_QUERY_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const sections = homeFeed.data?.sections ?? EMPTY_HOME_SECTIONS

  return (
    <main className="min-h-screen bg-background text-foreground">
      <div className="p-[32px_80px_16px_96px]">
        <div className="min-w-0 space-y-10">
          <FeedHeader title="首页" description="精选漫画作品" />

          {homeFeed.isLoading ? (
            <HomeFeedSkeleton />
          ) : homeFeed.isError ? (
            <StatePanel
              title="信息流加载失败"
              description={homeFeed.error.message}
              onRetry={() => homeFeed.refetch()}
            />
          ) : sections.length === 0 ? (
            <StatePanel title="暂无信息流内容" description="当前接口没有返回可展示的分组。" />
          ) : (
            <HomeFeedSections sections={sections} />
          )}
        </div>
      </div>
      {sections.length > 0 ? <HomeFeedDirectory sections={sections} /> : null}
      <BackTop />
    </main>
  )
}
