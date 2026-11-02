import { useMemo, useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ComicChapter, ComicDetail } from '@/domain/comic'
import { enqueueComicDownload, type DownloadChapterRequest } from '@/lib/api/download'
import { formatComicChapterTitle, SINGLE_CHAPTER_TITLE } from '@/lib/comic'
import { queryKeys } from '@/lib/query-keys'
import { markDownloadSubmitted } from '@/features/downloads/download-notifications'

export type DownloadChapterOption = DownloadChapterRequest

export function useComicDownload(comic: ComicDetail, sortedChapters: ComicChapter[]) {
  const queryClient = useQueryClient()
  const [isOpen, setIsOpen] = useState(false)
  const chapters = useMemo(
    () => toDownloadChapterOptions(comic.id, sortedChapters, comic.chapters),
    [comic.id, comic.chapters, sortedChapters]
  )
  const mutation = useMutation({
    mutationFn: (selectedChapters: DownloadChapterOption[]) =>
      enqueueComicDownload({
        albumId: comic.id,
        comicTitle: comic.title,
        comicAuthor: comic.authors.join(' / '),
        chapters: [...selectedChapters].sort((left, right) => left.order - right.order)
      }),
    onMutate: () => queryClient.cancelQueries({ queryKey: queryKeys.downloadTasks(), exact: true }),
    onSuccess: result => {
      markDownloadSubmitted(result.taskId)
      setIsOpen(false)
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '下载任务创建失败')
    },
    onSettled: async result => {
      await queryClient.cancelQueries({ queryKey: queryKeys.downloadTasks(), exact: true })
      if (result) queryClient.setQueryData(queryKeys.downloadTasks(), result)
      await queryClient.invalidateQueries({ queryKey: queryKeys.downloadTasks(), exact: true })
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

export function toDownloadChapterOptions(
  comicId: string,
  chapters: ComicChapter[],
  sourceChapters: ComicChapter[]
) {
  if (chapters.length === 0) {
    return [
      {
        chapterId: comicId,
        title: SINGLE_CHAPTER_TITLE,
        order: 1
      }
    ]
  }

  const readingOrder = [...sourceChapters]
  if (readingOrder.every(chapter => Number.isFinite(Number.parseInt(chapter.sort, 10)))) {
    readingOrder.sort(
      (left, right) => Number.parseInt(left.sort, 10) - Number.parseInt(right.sort, 10)
    )
  }
  const orders = new Map(readingOrder.map((chapter, index) => [chapter.id, index + 1]))

  return chapters.map((chapter, index) => ({
    chapterId: chapter.id,
    title: formatComicChapterTitle(chapter, index),
    order: orders.get(chapter.id) ?? index + 1
  }))
}
