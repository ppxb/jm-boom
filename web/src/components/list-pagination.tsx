import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious
} from '@/components/ui/pagination'

type PaginationEntry = number | 'start-ellipsis' | 'end-ellipsis'

export function ListPagination({
  page,
  hasMore,
  totalPages,
  disabled,
  onPageChange,
  scrollToTop = true
}: {
  page: number
  hasMore: boolean
  totalPages?: number
  disabled: boolean
  onPageChange: (page: number) => void
  scrollToTop?: boolean
}) {
  function changePage(nextPage: number) {
    if (disabled || nextPage < 1 || nextPage === page) {
      return
    }

    onPageChange(nextPage)
    if (scrollToTop) {
      window.scrollTo({ top: 0, behavior: 'smooth' })
    }
  }

  const exactPageCount = normalizeTotalPages(totalPages)
  const canGoNext = exactPageCount === undefined ? hasMore : page < exactPageCount
  const pageEntries = exactPageCount ? buildPaginationEntries(page, exactPageCount) : []

  return (
    <Pagination className="py-3">
      <PaginationContent>
        <PaginationItem>
          <PaginationPrevious
            href="#"
            text="上一页"
            aria-disabled={page <= 1 || disabled}
            className={page <= 1 || disabled ? 'pointer-events-none opacity-50' : undefined}
            onClick={event => {
              event.preventDefault()
              changePage(page - 1)
            }}
          />
        </PaginationItem>
        {exactPageCount ? (
          <PaginationItem className="sm:hidden">
            <span className="flex h-9 min-w-20 items-center justify-center px-2 text-sm tabular-nums">
              {page} / {exactPageCount}
            </span>
          </PaginationItem>
        ) : null}
        {pageEntries.map(entry => (
          <PaginationItem key={entry} className="hidden sm:block">
            {typeof entry === 'number' ? (
              <PaginationLink
                href="#"
                isActive={entry === page}
                aria-label={`第 ${entry} 页`}
                aria-disabled={disabled}
                className={disabled ? 'pointer-events-none opacity-50' : undefined}
                onClick={event => {
                  event.preventDefault()
                  changePage(entry)
                }}
              >
                {entry}
              </PaginationLink>
            ) : (
              <PaginationEllipsis />
            )}
          </PaginationItem>
        ))}
        <PaginationItem>
          <PaginationNext
            href="#"
            text="下一页"
            aria-disabled={disabled || !canGoNext}
            className={disabled || !canGoNext ? 'pointer-events-none opacity-50' : undefined}
            onClick={event => {
              event.preventDefault()
              changePage(page + 1)
            }}
          />
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  )
}

function normalizeTotalPages(totalPages: number | undefined) {
  if (!Number.isSafeInteger(totalPages) || totalPages === undefined || totalPages < 1) {
    return undefined
  }

  return totalPages
}

function buildPaginationEntries(page: number, totalPages: number): PaginationEntry[] {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, index) => index + 1)
  }

  if (page <= 4) {
    return [1, 2, 3, 4, 5, 'end-ellipsis', totalPages]
  }

  if (page >= totalPages - 3) {
    return [
      1,
      'start-ellipsis',
      totalPages - 4,
      totalPages - 3,
      totalPages - 2,
      totalPages - 1,
      totalPages
    ]
  }

  return [
    1,
    'start-ellipsis',
    page - 1,
    page,
    page + 1,
    'end-ellipsis',
    totalPages
  ]
}
