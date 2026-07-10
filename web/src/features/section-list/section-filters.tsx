import type { HomeSectionListMode } from '@/lib/api/home'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import {
  rankingCategoryOptions,
  RANKING_ORDER_OPTIONS,
  WEEK_CATEGORY_OPTIONS,
  WEEK_OPTIONS,
  type FilterOption
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
      <div className="flex justify-end gap-3">
        <FilterSelect
          value={week}
          options={WEEK_OPTIONS}
          placeholder="星期"
          onValueChange={onWeekChange}
        />
        <FilterSelect
          value={category}
          options={WEEK_CATEGORY_OPTIONS}
          placeholder="分类"
          onValueChange={onCategoryChange}
        />
      </div>
    )
  }

  if (mode === 'ranking') {
    const categoryOptions = rankingCategoryOptions(rankTag)

    return (
      <div className="flex justify-end gap-3">
        <FilterSelect
          value={order}
          options={RANKING_ORDER_OPTIONS}
          placeholder="排序"
          onValueChange={onOrderChange}
        />
        {categoryOptions.length > 1 ? (
          <FilterSelect
            value={category}
            options={categoryOptions}
            placeholder="分类"
            onValueChange={onCategoryChange}
          />
        ) : null}
      </div>
    )
  }

  return null
}

interface FilterSelectProps {
  value: string
  options: FilterOption[]
  placeholder: string
  onValueChange: (value: string) => void
}

function FilterSelect({ value, options, placeholder, onValueChange }: FilterSelectProps) {
  return (
    <Select value={value} onValueChange={onValueChange}>
      <SelectTrigger>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          {options.map(option => (
            <SelectItem key={option.value} value={option.value}>
              {option.label}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  )
}
