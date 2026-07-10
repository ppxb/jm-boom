import { useQuery } from '@tanstack/react-query'
import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { ListFilterIcon, SearchIcon } from 'lucide-react'
import { FormEvent, useEffect, useState } from 'react'

import { ComicGrid, ComicGridSkeleton } from '@/components/comic'
import { EmptyState } from '@/components/empty-state'
import { ListPagination } from '@/components/list-pagination'
import { PageBackButton } from '@/components/page-back-button'
import { Button } from '@/components/ui/button'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput
} from '@/components/ui/input-group'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select'
import { searchComic, type ComicListItem } from '@/lib/api/search'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'

type SearchPageSearch = {
  q?: string
  page?: number
  sort?: SearchSortBy
}

type SearchSortBy = 1 | 2 | 3 | 4

export const Route = createFileRoute('/_app/explore/search')({
  validateSearch: validateSearchParams,
  component: SearchPage
})

const SEARCH_SORT_OPTIONS = [
  { label: '从新到旧', value: '1' },
  { label: '最多观看', value: '2' },
  { label: '最多图片', value: '3' },
  { label: '最多点赞', value: '4' }
] as const

function SearchPage() {
  const navigate = useNavigate({ from: Route.fullPath })
  const search = Route.useSearch()
  const keyword = search.q ?? ''
  const page = search.page ?? 1
  const sortBy = search.sort ?? 1
  const [draftKeyword, setDraftKeyword] = useState(keyword)

  useEffect(() => {
    setDraftKeyword(keyword)
  }, [keyword])

  const query = useQuery({
    queryKey: queryKeys.search(keyword, page, sortBy),
    queryFn: () =>
      searchComic({
        keyword,
        page,
        extern: { sortBy }
      }),
    enabled: keyword.length > 0,
    staleTime: CACHE.LIST_STALE_TIME,
    gcTime: CACHE.LIST_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false
  })
  const items = mapSearchItems(query.data?.items ?? [])
  const paging = query.data?.paging

  function submitSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const nextKeyword = draftKeyword.trim()

    void navigate({
      search: createSearchParams({ q: nextKeyword, sort: sortBy })
    })
  }

  function updateSortBy(value: string) {
    const nextSort = parseSortBy(value)

    void navigate({
      replace: true,
      resetScroll: false,
      search: createSearchParams({ q: keyword, sort: nextSort })
    })
  }

  function updatePage(page: number) {
    void navigate({
      replace: true,
      resetScroll: false,
      search: createSearchParams({ q: keyword, page, sort: sortBy })
    })
  }

  return (
    <section className="space-y-6">
      <PageBackButton />

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <form className="w-full sm:max-w-xl" onSubmit={submitSearch}>
          <InputGroup className="h-10">
            <InputGroupAddon>
              <SearchIcon className="size-4" />
            </InputGroupAddon>
            <InputGroupInput
              value={draftKeyword}
              onChange={event => setDraftKeyword(event.target.value)}
              placeholder="搜索关键词或 JM 号"
              aria-label="搜索关键词"
            />
            <InputGroupAddon align="inline-end">
              <InputGroupButton type="submit" variant="secondary" size="sm">
                搜索
              </InputGroupButton>
            </InputGroupAddon>
          </InputGroup>
        </form>

        <Select value={String(sortBy)} onValueChange={updateSortBy} disabled={keyword.length === 0}>
          <SelectTrigger className="w-full sm:w-auto">
            <ListFilterIcon className="size-4 text-muted-foreground" />
            <SelectValue placeholder="选择排序" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {SEARCH_SORT_OPTIONS.map(option => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <SearchContent
        keyword={keyword}
        isError={query.isError}
        isLoading={query.isLoading}
        items={items}
        page={page}
        hasMore={!paging?.hasReachedMax}
        disabled={query.isFetching}
        onRetry={() => query.refetch()}
        onPageChange={updatePage}
      />
    </section>
  )
}

function SearchContent({
  keyword,
  isError,
  isLoading,
  items,
  page,
  hasMore,
  disabled,
  onRetry,
  onPageChange
}: {
  keyword: string
  isError: boolean
  isLoading: boolean
  items: ReturnType<typeof mapSearchItems>
  page: number
  hasMore: boolean
  disabled: boolean
  onRetry: () => void
  onPageChange: (page: number) => void
}) {
  if (keyword.length === 0) {
    return null // 不显示任何提示
  }

  if (isError) {
    return (
      <EmptyState
        emoji="Ò︵Ó"
        title="数据加载失败"
        actions={
          <Button type="button" variant="outline" size="sm" onClick={onRetry}>
            重试
          </Button>
        }
      />
    )
  }

  if (isLoading) {
    return <ComicGridSkeleton count={12} />
  }

  if (items.length === 0) {
    return <EmptyState emoji="(･o･;)" title="没有搜索结果" />
  }

  return (
    <>
      <ComicGrid items={items} />
      <ListPagination
        page={page}
        hasMore={hasMore}
        disabled={disabled}
        onPageChange={onPageChange}
      />
    </>
  )
}

function mapSearchItems(items: ComicListItem[]) {
  return items.map(item => ({
    id: item.id,
    title: item.title,
    author: searchItemAuthor(item),
    description: String(item.raw.description ?? ''),
    image: item.cover.url,
    tags: item.metadata
      .filter(meta => meta.type !== 'author')
      .flatMap(meta => meta.value)
      .filter(Boolean),
    updatedAt: Number.parseInt(item.updatedAt, 10) || null
  }))
}

function searchItemAuthor(item: ComicListItem) {
  const authorMeta = item.metadata.find(meta => meta.type === 'author')
  const author = authorMeta?.value.join(' / ').trim()

  return author || String(item.raw.author ?? '')
}

function validateSearchParams(search: Record<string, unknown>): SearchPageSearch {
  return createSearchParams({
    q: typeof search.q === 'string' ? search.q : '',
    page: parseOptionalPage(search.page),
    sort: parseSortBy(search.sort)
  })
}

function createSearchParams({
  q,
  page = 1,
  sort = 1
}: {
  q: string
  page?: number
  sort?: SearchSortBy
}): SearchPageSearch {
  const keyword = q.trim()

  if (keyword.length === 0) {
    return {}
  }

  return {
    q: keyword,
    ...(page > 1 ? { page } : {}),
    ...(sort !== 1 ? { sort } : {})
  }
}

function parseOptionalPage(value: unknown) {
  const page = Number(value)

  return Number.isSafeInteger(page) && page > 1 ? page : undefined
}

function parseSortBy(value: unknown): SearchSortBy {
  const sortBy = Number(value)

  return sortBy === 2 || sortBy === 3 || sortBy === 4 ? sortBy : 1
}
