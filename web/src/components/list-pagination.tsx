import {
  Pagination,
  PaginationContent,
  PaginationItem,
  PaginationNext,
  PaginationPrevious
} from '@/components/ui/pagination'

export function ListPagination({
  page,
  hasMore,
  disabled,
  onPageChange,
  scrollToTop = true
}: {
  page: number
  hasMore: boolean
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
        <PaginationItem>
          <span className="flex h-9 min-w-20 items-center justify-center px-2 text-sm tabular-nums">
            第 {page} 页
          </span>
        </PaginationItem>
        <PaginationItem>
          <PaginationNext
            href="#"
            text="下一页"
            aria-disabled={disabled || !hasMore}
            className={disabled || !hasMore ? 'pointer-events-none opacity-50' : undefined}
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
