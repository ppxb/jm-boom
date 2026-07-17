import { Link } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useEffect, useMemo, useRef } from 'react'

import { SideDrawerContent } from '@/components/side-drawer-content'
import { Badge } from '@/components/ui/badge'
import { Drawer, DrawerDescription, DrawerHeader, DrawerTitle } from '@/components/ui/drawer'
import { cn } from '@/lib/utils'
import { listDownloadedChapters } from '@/lib/api/download'
import { queryKeys } from '@/lib/query-keys'
import { toReaderChapterSearch } from './reader-chapter-link'
import type { ReaderChapterItem } from './types'

export function ReaderChapterDrawer({
  open,
  onOpenChange,
  title,
  albumId,
  currentReadId,
  chapters
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  albumId: string
  currentReadId: string
  chapters: ReaderChapterItem[]
}) {
  const listRef = useRef<HTMLDivElement | null>(null)
  const displayChapters = useMemo(() => [...chapters].reverse(), [chapters])
  const downloadedChapters = useQuery({
    queryKey: queryKeys.downloadedChapters(),
    queryFn: listDownloadedChapters,
    enabled: open,
    staleTime: 5000,
    refetchInterval: open ? 2000 : false,
    refetchOnMount: 'always',
    refetchOnWindowFocus: false
  })
  const downloadedChapterIds = useMemo(
    () => new Set(downloadedChapters.data?.chapterIds ?? []),
    [downloadedChapters.data?.chapterIds]
  )

  useEffect(() => {
    if (!open) {
      return
    }

    const timeoutId = window.setTimeout(() => {
      listRef.current?.querySelector<HTMLElement>('[data-current-chapter="true"]')?.scrollIntoView({
        behavior: 'smooth',
        block: 'center'
      })
    }, 180)

    return () => window.clearTimeout(timeoutId)
  }, [currentReadId, displayChapters.length, open])

  return (
    <Drawer open={open} onOpenChange={onOpenChange} direction="right">
      <SideDrawerContent>
        <DrawerHeader className="border-b border-border/70">
          <DrawerTitle>章节目录</DrawerTitle>
          <DrawerDescription className="line-clamp-1">{title || '当前作品'}</DrawerDescription>
        </DrawerHeader>

        <nav className="min-h-0 flex-1 scroll-fade-y overflow-y-auto px-4 py-3">
          <div ref={listRef} className="space-y-2">
            {displayChapters.map(chapter => {
              const isCurrent = chapter.id === currentReadId
              const isDownloaded = downloadedChapterIds.has(chapter.id)

              return (
                <Link
                  key={chapter.id}
                  to="/reader/$comicId"
                  params={{ comicId: chapter.id }}
                  replace
                  resetScroll
                  search={toReaderChapterSearch({ albumId })}
                  aria-current={isCurrent ? 'page' : undefined}
                  data-current-chapter={isCurrent ? 'true' : undefined}
                  className={cn(
                    'flex items-center justify-between gap-3 rounded-md border border-border bg-card/60 px-3 py-2.5 text-sm text-card-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none',
                    isCurrent && 'border-primary/40 bg-accent text-accent-foreground'
                  )}
                  onClick={() => onOpenChange(false)}
                >
                  <span className="min-w-0 truncate font-medium">{chapter.title}</span>
                  <span className="flex shrink-0 items-center gap-1.5">
                    {isDownloaded ? (
                      <Badge
                        variant="outline"
                        className="border-emerald-600/30 bg-emerald-500/10 text-emerald-700 dark:border-emerald-400/30 dark:text-emerald-300"
                      >
                        已下载
                      </Badge>
                    ) : null}
                    <Badge
                      variant="outline"
                      className="border-border bg-muted/60 text-muted-foreground"
                    >
                      {isCurrent ? '当前' : `JM ${chapter.id}`}
                    </Badge>
                  </span>
                </Link>
              )
            })}
          </div>
        </nav>
      </SideDrawerContent>
    </Drawer>
  )
}
