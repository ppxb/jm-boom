import { Link } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { XIcon } from 'lucide-react'
import { useEffect, useMemo, useRef } from 'react'

import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle
} from '@/components/ui/drawer'
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
      <DrawerContent className="h-full w-[82vw] max-w-[280px] overflow-hidden rounded-l-xl border-l border-white/10 bg-neutral-950/90 p-0 text-neutral-50 shadow-2xl backdrop-blur-xl before:hidden data-[vaul-drawer-direction=right]:w-[82vw] data-[vaul-drawer-direction=right]:max-w-[280px] data-[vaul-drawer-direction=right]:sm:w-[420px] data-[vaul-drawer-direction=right]:sm:max-w-[420px]">
        <DrawerHeader className="relative border-b border-white/10 pr-12">
          <DrawerTitle className="text-neutral-50">章节目录</DrawerTitle>
          <DrawerDescription className="line-clamp-1 text-neutral-400">
            {title || '当前作品'}
          </DrawerDescription>
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            aria-label="关闭章节目录"
            className="absolute top-4 right-4 text-neutral-300 hover:bg-white/10 hover:text-neutral-50 focus-visible:text-neutral-50"
            onClick={() => onOpenChange(false)}
          >
            <XIcon className="size-4" />
          </Button>
        </DrawerHeader>

        <nav className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
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
                  search={toReaderChapterSearch({ albumId })}
                  aria-current={isCurrent ? 'page' : undefined}
                  data-current-chapter={isCurrent ? 'true' : undefined}
                  className={cn(
                    'flex items-center justify-between gap-3 rounded-md border border-white/10 bg-white/5 px-3 py-2.5 text-sm text-neutral-200 transition-colors hover:bg-white/10 hover:text-neutral-50 focus-visible:ring-[3px] focus-visible:ring-white/20 focus-visible:outline-none',
                    isCurrent && 'border-neutral-50/35 bg-white/10 text-neutral-50'
                  )}
                  onClick={() => onOpenChange(false)}
                >
                  <span className="min-w-0 truncate font-medium">{chapter.title}</span>
                  <span className="flex shrink-0 items-center gap-1.5">
                    {isDownloaded ? (
                      <Badge
                        variant="outline"
                        className="border-emerald-400/30 bg-emerald-400/10 text-emerald-300"
                      >
                        已下载
                      </Badge>
                    ) : null}
                    <Badge
                      variant="outline"
                      className="border-white/10 bg-white/5 text-neutral-400"
                    >
                      {isCurrent ? '当前' : `JM ${chapter.id}`}
                    </Badge>
                  </span>
                </Link>
              )
            })}
          </div>
        </nav>
      </DrawerContent>
    </Drawer>
  )
}
