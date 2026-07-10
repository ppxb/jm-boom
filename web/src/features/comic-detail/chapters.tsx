import { Link } from '@tanstack/react-router'
import { useEffect, useMemo, useState } from 'react'

import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'
import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious
} from '@/components/ui/pagination'
import type { ComicChapter } from '@/lib/api/comic'
import {
  SINGLE_CHAPTER_TITLE,
  formatComicChapterTitle,
  getComicDisplayChapterCount,
  sortComicChapters
} from '@/lib/comic'
import { UI } from '@/lib/constants'
import { cn } from '@/lib/utils'
import { SectionHeading } from './shared'

export function ChaptersSection({
  albumId,
  comicId,
  chapters
}: {
  albumId: string
  comicId: string
  chapters: ComicChapter[]
}) {
  const sortedChapters = useMemo(() => sortComicChapters(chapters), [chapters])
  const displayChapterCount = getComicDisplayChapterCount(chapters)
  const [page, setPage] = useState(1)
  const pageCount = Math.max(1, Math.ceil(sortedChapters.length / UI.CHAPTER_PAGE_SIZE))
  const safePage = Math.min(page, pageCount)
  const visibleChapters = sortedChapters.slice(
    (safePage - 1) * UI.CHAPTER_PAGE_SIZE,
    safePage * UI.CHAPTER_PAGE_SIZE
  )

  useEffect(() => {
    setPage(current => Math.min(current, pageCount))
  }, [pageCount])

  function changePage(nextPage: number) {
    const clampedPage = Math.min(Math.max(nextPage, 1), pageCount)
    setPage(clampedPage)
    document.getElementById('chapters')?.scrollIntoView({
      behavior: 'smooth',
      block: 'start'
    })
  }

  return (
    <section id="chapters" className="scroll-mt-8 space-y-4">
      <SectionHeading title="章节" description={`${displayChapterCount} 个章节`} />
      {sortedChapters.length === 0 ? (
        <Link
          to="/reader/$comicId"
          params={{ comicId }}
          search={{
            albumId
          }}
          className="block"
        >
          <Card size="sm" className="py-0 transition-colors hover:bg-muted/40">
            <CardContent className="flex items-center justify-between gap-4 p-4">
              <div className="min-w-0">
                <div className="truncate text-sm font-medium">{SINGLE_CHAPTER_TITLE}</div>
                <div className="text-xs text-muted-foreground">单行本</div>
              </div>
              <Badge variant="outline">JM {comicId}</Badge>
            </CardContent>
          </Card>
        </Link>
      ) : (
        <>
          <div className="space-y-2">
            {visibleChapters.map((chapter, index) => {
              const chapterIndex = (safePage - 1) * UI.CHAPTER_PAGE_SIZE + index
              const chapterTitle = formatComicChapterTitle(chapter, chapterIndex)

              return (
                <Link
                  key={chapter.id}
                  to="/reader/$comicId"
                  params={{ comicId: chapter.id }}
                  search={{
                    albumId
                  }}
                  className="block"
                >
                  <Card size="sm" className="py-0 transition-colors hover:bg-muted/40">
                    <CardContent className="flex items-center justify-between gap-4 p-4">
                      <div className="min-w-0">
                        <div className="truncate text-sm font-medium">{chapterTitle}</div>
                        <div className="text-xs text-muted-foreground">
                          {chapter.sort
                            ? `第 ${chapter.sort} 章`
                            : `章节 ${(safePage - 1) * UI.CHAPTER_PAGE_SIZE + index + 1}`}
                        </div>
                      </div>
                      <Badge variant="outline">JM {chapter.id}</Badge>
                    </CardContent>
                  </Card>
                </Link>
              )
            })}
          </div>

          {pageCount > 1 ? (
            <ChapterPagination page={safePage} pageCount={pageCount} onPageChange={changePage} />
          ) : null}
        </>
      )}
    </section>
  )
}

function getVisiblePages(currentPage: number, pageCount: number) {
  if (pageCount <= 7) {
    return Array.from({ length: pageCount }, (_, index) => index + 1)
  }

  const pages = new Set([1, pageCount, currentPage - 1, currentPage, currentPage + 1])
  const sortedPages = [...pages]
    .filter(page => page >= 1 && page <= pageCount)
    .sort((left, right) => left - right)
  const visiblePages: Array<number | 'ellipsis'> = []

  for (const page of sortedPages) {
    const previousPage = visiblePages[visiblePages.length - 1]

    if (typeof previousPage === 'number' && page - previousPage > 1) {
      visiblePages.push('ellipsis')
    }

    visiblePages.push(page)
  }

  return visiblePages
}

function ChapterPagination({
  page,
  pageCount,
  onPageChange
}: {
  page: number
  pageCount: number
  onPageChange: (page: number) => void
}) {
  const pages = getVisiblePages(page, pageCount)

  return (
    <Pagination className="pt-2">
      <PaginationContent>
        <PaginationItem>
          <PaginationPrevious
            href="#"
            text="上一页"
            className={cn(page === 1 && 'pointer-events-none opacity-50')}
            onClick={event => {
              event.preventDefault()
              onPageChange(page - 1)
            }}
          />
        </PaginationItem>
        {pages.map((item, index) =>
          item === 'ellipsis' ? (
            <PaginationItem key={`ellipsis-${index}`}>
              <PaginationEllipsis />
            </PaginationItem>
          ) : (
            <PaginationItem key={item}>
              <PaginationLink
                href="#"
                isActive={item === page}
                onClick={event => {
                  event.preventDefault()
                  onPageChange(item)
                }}
              >
                {item}
              </PaginationLink>
            </PaginationItem>
          )
        )}
        <PaginationItem>
          <PaginationNext
            href="#"
            text="下一页"
            className={cn(page === pageCount && 'pointer-events-none opacity-50')}
            onClick={event => {
              event.preventDefault()
              onPageChange(page + 1)
            }}
          />
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  )
}
