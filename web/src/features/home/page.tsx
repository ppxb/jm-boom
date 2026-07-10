import { useQuery } from '@tanstack/react-query'

import { EmptyState } from '@/components/empty-state'
import { Button } from '@/components/ui/button'
import { getHomeFeed, type HomeFeedSection } from '@/lib/api/home'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'
import { HomeFeedSections } from './home-feed-sections'
import { HomeFeedSkeleton } from './home-feed-skeleton'

const EMPTY_HOME_SECTIONS: HomeFeedSection[] = []

export function HomePage() {
  const homeFeed = useQuery({
    queryKey: queryKeys.homeFeed(),
    queryFn: getHomeFeed,
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const sections = homeFeed.data?.sections ?? EMPTY_HOME_SECTIONS

  return (
    <div className="min-w-0 space-y-8">
      {homeFeed.isLoading ? (
        <HomeFeedSkeleton />
      ) : homeFeed.isError ? (
        <EmptyState
          emoji="Ò︵Ó"
          title="数据加载失败"
          actions={
            <Button type="button" variant="outline" size="sm" onClick={() => homeFeed.refetch()}>
              重试
            </Button>
          }
        />
      ) : sections.length === 0 ? (
        <EmptyState emoji="(･o･;)" title="暂无信息流内容" />
      ) : (
        <HomeFeedSections sections={sections} />
      )}
    </div>
  )
}
