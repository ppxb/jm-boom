import { Link } from '@tanstack/react-router'
import { ArrowRightIcon } from 'lucide-react'

import { ComicGrid, StatePanel } from '@/components/comic-feed'
import { Button } from '@/components/ui/button'
import type { HomeFeedSection } from '@/lib/api/home'
import { defaultRankingCategory } from '@/lib/ranking-filters'
import { currentChinaWeekday, homeSectionId } from './home-utils'

export function HomeFeedSections({ sections }: { sections: HomeFeedSection[] }) {
  return (
    <>
      {sections.map(section => (
        <section key={section.id} id={homeSectionId(section)} className="scroll-mt-8 space-y-6">
          <SectionHeader section={section} />
          {section.items.length === 0 ? (
            <StatePanel title="暂无内容" description="当前分组没有返回可展示的漫画。" />
          ) : (
            <ComicGrid items={section.items} />
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
            to="/list"
            search={{
              mode,
              page: 1,
              sectionId: section.id,
              title: section.title,
              slug: section.slug,
              type: section.type,
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
