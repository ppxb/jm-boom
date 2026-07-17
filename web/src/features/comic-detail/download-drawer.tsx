import { DownloadIcon } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'

import { SideDrawerContent } from '@/components/side-drawer-content'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Drawer,
  DrawerDescription,
  DrawerFooter,
  DrawerHeader,
  DrawerTitle
} from '@/components/ui/drawer'
import type { ComicChapter } from '@/domain/comic'
import type { DownloadChapterRequest } from '@/lib/api/download'
import { formatComicChapterTitle } from '@/lib/comic'

export type DownloadChapterOption = DownloadChapterRequest

export function toDownloadChapterOptions(chapters: ComicChapter[]) {
  return chapters.map((chapter, index) => ({
    chapterId: chapter.id,
    title: formatComicChapterTitle(chapter, index),
    order: index + 1
  }))
}

export function ComicDownloadDrawer({
  open,
  onOpenChange,
  comicTitle,
  chapters,
  isSubmitting,
  onConfirm
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  comicTitle: string
  chapters: DownloadChapterOption[]
  isSubmitting: boolean
  onConfirm: (chapters: DownloadChapterOption[]) => void
}) {
  const [selectedChapterIds, setSelectedChapterIds] = useState<Set<string>>(() => new Set())
  const selectedChapters = useMemo(
    () => chapters.filter(chapter => selectedChapterIds.has(chapter.chapterId)),
    [chapters, selectedChapterIds]
  )
  const allSelected = chapters.length > 0 && selectedChapterIds.size === chapters.length

  useEffect(() => {
    if (open) {
      setSelectedChapterIds(new Set())
    }
  }, [open])

  function toggleChapter(chapterId: string, checked: boolean) {
    setSelectedChapterIds(current => {
      const next = new Set(current)

      if (checked) {
        next.add(chapterId)
      } else {
        next.delete(chapterId)
      }

      return next
    })
  }

  function toggleAll(checked: boolean) {
    setSelectedChapterIds(checked ? new Set(chapters.map(chapter => chapter.chapterId)) : new Set())
  }

  return (
    <Drawer open={open} onOpenChange={onOpenChange} direction="right">
      <SideDrawerContent>
        <DrawerHeader>
          <DrawerTitle>选择下载章节</DrawerTitle>
          <DrawerDescription className="line-clamp-2">{comicTitle}</DrawerDescription>
        </DrawerHeader>

        <div className="border-y border-border/70 px-4 py-3">
          <div className="flex items-center justify-between gap-3 text-sm">
            <label className="flex cursor-pointer items-center gap-3">
              <Checkbox
                checked={allSelected}
                onCheckedChange={checked => toggleAll(checked === true)}
              />
              全选
            </label>
            <Badge variant="outline">
              已选择 {selectedChapters.length} / {chapters.length}
            </Badge>
          </div>
        </div>

        <div className="min-h-0 flex-1 scroll-fade-y overflow-y-auto px-4 py-3">
          <div className="space-y-2">
            {chapters.map(chapter => {
              const checked = selectedChapterIds.has(chapter.chapterId)

              return (
                <label
                  key={chapter.chapterId}
                  className="flex cursor-pointer items-center gap-3 rounded-md border border-border/70 bg-card/60 p-3 transition-colors hover:bg-muted/50"
                >
                  <Checkbox
                    checked={checked}
                    onCheckedChange={nextChecked =>
                      toggleChapter(chapter.chapterId, nextChecked === true)
                    }
                  />
                  <div className="min-w-0 flex-1 truncate text-sm font-medium">{chapter.title}</div>
                  <Badge variant="outline" className="shrink-0">
                    JM {chapter.chapterId}
                  </Badge>
                </label>
              )
            })}
          </div>
        </div>

        <DrawerFooter className="border-t border-border/70">
          <Button
            disabled={selectedChapters.length === 0 || isSubmitting}
            onClick={() => onConfirm(selectedChapters)}
          >
            <DownloadIcon className="size-4" />
            下载选中章节
          </Button>
        </DrawerFooter>
      </SideDrawerContent>
    </Drawer>
  )
}
