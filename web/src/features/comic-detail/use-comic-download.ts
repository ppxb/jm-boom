import { useMemo, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ComicChapter, ComicDetail } from '@/domain/comic'
import { enqueueComicDownload, type DownloadChapterRequest } from '@/lib/api/download'
import { formatComicChapterTitle, SINGLE_CHAPTER_TITLE } from '@/lib/comic'
import { queryKeys } from '@/lib/query-keys'

export type DownloadChapterOption = DownloadChapterRequest

export function useComicDownload(comic: ComicDetail, sortedChapters: ComicChapter[]) {
  const queryClient = useQueryClient()
  const [isOpen, setIsOpen] = useState(false)
  const chapters = useMemo(
    () => toDownloadChapterOptions(comic.id, sortedChapters),
    [comic.id, sortedChapters]
  )
  const mutation = useMutation({
    mutationFn: (selectedChapters: DownloadChapterOption[]) =>
      enqueueComicDownload({
        albumId: comic.id,
        comicTitle: comic.title,
        chapters: selectedChapters
      }),
    onSuccess: result => {
      queryClient.setQueryData(queryKeys.downloadTasks(), result)
      setIsOpen(false)
      toast.success('已加入下载队列，可在下载页查看进度')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '下载任务创建失败')
    }
  })

  function start() {
    if (chapters.length <= 1) {
      mutation.mutate(chapters)
      return
    }

    setIsOpen(true)
  }

  return {
    chapters,
    isOpen,
    setIsOpen,
    isPending: mutation.isPending,
    start,
    submit: mutation.mutate
  }
}

function toDownloadChapterOptions(comicId: string, chapters: ComicChapter[]) {
  if (chapters.length === 0) {
    return [
      {
        chapterId: comicId,
        title: SINGLE_CHAPTER_TITLE,
        order: 1
      }
    ]
  }

  return chapters.map((chapter, index) => ({
    chapterId: chapter.id,
    title: formatComicChapterTitle(chapter, index),
    order: index + 1
  }))
}
