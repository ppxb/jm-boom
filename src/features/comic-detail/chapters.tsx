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
import { cn } from '@/lib/utils'
import { CHAPTER_PAGE_SIZE } from './constants'
import { SectionHeading, StatePanel } from './shared'
import { formatChapterTitle, getVisiblePages, sortChapters } from './utils'

export function ChaptersSection({
  albumId,
  comicTitle,
  chapters
}: {
  albumId: string
  comicTitle: string
  chapters: ComicChapter[]
}) {
  const sortedChapters = useMemo(() => sortChapters(chapters), [chapters])
  const [page, setPage] = useState(1)
  const pageCount = Math.max(1, Math.ceil(sortedChapters.length / CHAPTER_PAGE_SIZE))
  const safePage = Math.min(page, pageCount)
  const visibleChapters = sortedChapters.slice(
    (safePage - 1) * CHAPTER_PAGE_SIZE,
    safePage * CHAPTER_PAGE_SIZE
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
      <SectionHeading
        title="章节"
        description={`${chapters.length} 个章节${pageCount > 1 ? `，第 ${safePage}/${pageCount} 页` : ''}`}
      />
      {sortedChapters.length === 0 ? (
        <StatePanel title="暂无章节" description="当前作品可能是单行本。" />
      ) : (
        <>
          <div className="space-y-2">
            {visibleChapters.map((chapter, index) => {
              const chapterIndex = (safePage - 1) * CHAPTER_PAGE_SIZE + index
              const nextChapter = sortedChapters[chapterIndex - 1] ?? null
              const chapterTitle = formatChapterTitle(chapter, chapterIndex)
              const nextChapterTitle = nextChapter
                ? formatChapterTitle(nextChapter, chapterIndex - 1)
                : ''

              return (
                <Link
                  key={chapter.id}
                  to="/reader/$comicId"
                  params={{ comicId: chapter.id }}
                  search={{
                    title: comicTitle,
                    chapter: chapterTitle,
                    albumId,
                    fromDetail: '1',
                    pageIndex: '0',
                    nextId: nextChapter?.id ?? '',
                    nextChapter: nextChapterTitle
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
                            : `章节 ${(safePage - 1) * CHAPTER_PAGE_SIZE + index + 1}`}
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
