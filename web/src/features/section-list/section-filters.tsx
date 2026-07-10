import type { HomeSectionListMode } from '@/lib/api/home'
import { BarChart3Icon, CalendarDaysIcon, ListFilterIcon, TagsIcon } from 'lucide-react'

import { FilterSelect } from '@/components/filter-select'
import {
  rankingCategoryOptions,
  RANKING_ORDER_OPTIONS,
  WEEK_CATEGORY_OPTIONS,
  WEEK_OPTIONS
} from '@/lib/filters'

interface SectionFiltersProps {
  mode: HomeSectionListMode
  rankTag: string
  category: string
  week: string
  order: string
  onCategoryChange: (value: string) => void
  onWeekChange: (value: string) => void
  onOrderChange: (value: string) => void
}

export function SectionFilters({
  mode,
  rankTag,
  category,
  week,
  order,
  onCategoryChange,
  onWeekChange,
  onOrderChange
}: SectionFiltersProps) {
  if (mode === 'weekly') {
    return (
      <div className="flex items-center justify-end gap-2">
        <FilterSelect
          value={week}
          options={WEEK_OPTIONS}
          placeholder="星期"
          icon={<CalendarDaysIcon className="size-4 text-muted-foreground" />}
          onValueChange={onWeekChange}
        />
        <FilterSelect
          value={category}
          options={WEEK_CATEGORY_OPTIONS}
          placeholder="分类"
          icon={<TagsIcon className="size-4 text-muted-foreground" />}
          onValueChange={onCategoryChange}
        />
      </div>
    )
  }

  if (mode === 'ranking') {
    const categoryOptions = rankingCategoryOptions(rankTag)
    const hasCategoryFilter = categoryOptions.length > 1

    return (
      <div className="flex items-center justify-end gap-2">
        <FilterSelect
          value={order}
          options={RANKING_ORDER_OPTIONS}
          placeholder="排序"
          grow={hasCategoryFilter}
          icon={<ListFilterIcon className="size-4 text-muted-foreground" />}
          onValueChange={onOrderChange}
        />
        {hasCategoryFilter ? (
          <FilterSelect
            value={category}
            options={categoryOptions}
            placeholder="分类"
            icon={<BarChart3Icon className="size-4 text-muted-foreground" />}
            onValueChange={onCategoryChange}
          />
        ) : null}
      </div>
    )
  }

  return null
}
