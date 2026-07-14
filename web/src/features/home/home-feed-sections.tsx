import { Link } from '@tanstack/react-router'
import { ArrowRightIcon } from 'lucide-react'

import { ComicCard, ComicRail, ComicRailItem } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { Button } from '@/components/ui/button'
import type { HomeFeedSection } from '@/lib/api/home'
import { currentChinaWeekday } from '@/lib/utils'
import { defaultRankingCategory } from '@/lib/filters'
import { homeSectionId } from './home-utils'

export function HomeFeedSections({ sections }: { sections: HomeFeedSection[] }) {
  return (
    <>
      {sections.map(section => (
        <section
          key={section.id}
          id={homeSectionId(section)}
          className="min-w-0 scroll-mt-8 space-y-6"
        >
          <SectionHeader section={section} />
          {section.items.length === 0 ? (
            <EmptyState emoji="(･o･;)" title="暂无内容" />
          ) : (
            <ComicRail>
              {section.items.map(item => (
                <ComicRailItem key={item.id}>
                  <ComicCard
                    comic={item}
                    ratio="square"
                    showIdBadge
                    linkProps={{
                      to: '/comic/$comicId',
                      params: { comicId: item.id }
                    }}
                    metadata={
                      <p className="line-clamp-1 text-xs text-muted-foreground">
                        {item.author || 'N/A'}
                      </p>
                    }
                  />
                </ComicRailItem>
              ))}
            </ComicRail>
          )}
        </section>
      ))}
    </>
  )
}

function SectionHeader({ section }: { section: HomeFeedSection }) {
  const mode = section.listMode
  const rankTag = mode === 'ranking' ? section.rankTag : ''

  return (
    <div className="flex items-end justify-between gap-3">
      <div className="space-y-1">
        <h2 className="text-xl font-semibold tracking-normal">{section.title}</h2>
      </div>
      {mode ? (
        <Button asChild variant="outline" size="sm">
          <Link
            to="/explore/list"
            search={{
              mode,
              page: 1,
              sectionId: section.id,
              title: section.title,
              filterValue: section.filterValue,
              rankTag,
              category: mode === 'ranking' ? defaultRankingCategory(rankTag) : 'all',
              week: String(currentChinaWeekday()),
              order: 'new'
            }}
          >
            更多
            <ArrowRightIcon className="size-4" />
          </Link>
        </Button>
      ) : null}
    </div>
  )
}
