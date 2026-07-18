import { useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'

import type { ComicDetail } from '@/domain/comic'
import type { ComicStateResult } from '@/lib/api/comic'
import { addFavorite, removeFavorite } from '@/lib/api/favorite'
import { queryKeys } from '@/lib/query-keys'

export function useComicFavorite({
  comic,
  state,
  stateLoading
}: {
  comic: ComicDetail
  state: ComicStateResult | undefined
  stateLoading: boolean
}) {
  const queryClient = useQueryClient()
  const isFavorite = state?.isFavorite ?? false
  const mutation = useMutation({
    mutationFn: async () => {
      if (isFavorite) {
        await removeFavorite(comic.id)
        return { isFavorite: false as const }
      }

      await addFavorite({
        id: comic.id,
        title: comic.title,
        author: comic.authors.join(' / '),
        description: comic.description,
        image: comic.image,
        tags: comic.tags
      })
      return { isFavorite: true as const }
    },
    onSuccess: result => {
      queryClient.setQueryData<ComicStateResult>(queryKeys.comicState(comic.id), current => ({
        isFavorite: result.isFavorite,
        history: current?.history ?? null
      }))
      void queryClient.invalidateQueries({ queryKey: queryKeys.favorites() })
      toast.success(result.isFavorite ? '已添加收藏' : '已取消收藏')
    },
    onError: error => {
      toast.error(error instanceof Error ? error.message : '收藏操作失败')
    }
  })

  function toggle() {
    mutation.mutate()
  }

  return {
    isFavorite,
    isPending: stateLoading || mutation.isPending,
    toggle
  }
}
