import { useQuery, useQueryClient } from '@tanstack/react-query'
import { BookOpenIcon, UserRoundIcon } from 'lucide-react'

import { BackTopButton } from '@/components/back-top-button'
import { ComicCover } from '@/components/comic/comic-cover'
import { EmptyState } from '@/components/empty-state'
import { PageBackButton } from '@/components/page-back-button'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import {
  createSourceMangaStub,
  getSourceManga,
  type SourceChapter,
  type SourceManga
} from '@/lib/api/source'
import { CACHE } from '@/lib/constants'
import { queryKeys } from '@/lib/query-keys'

export function SourceComicDetailPage({
  sourceId,
  mangaKey
}: {
  sourceId: string
  mangaKey: string
}) {
  const queryClient = useQueryClient()
  const queryKey = queryKeys.sourceManga(sourceId, mangaKey)
  const seed = queryClient.getQueryData<SourceManga>(queryKey) ?? createSourceMangaStub(mangaKey)
  const detail = useQuery({
    queryKey,
    queryFn: () => getSourceManga(sourceId, seed),
    staleTime: 0,
    gcTime: CACHE.DETAIL_GC_TIME,
    initialData: seed,
    initialDataUpdatedAt: 0,
    refetchOnMount: true,
    refetchOnWindowFocus: false
  })

  return (
    <main className="min-h-screen bg-background px-4 py-6 text-foreground sm:px-6 lg:px-8">
      <div className="mx-auto max-w-7xl space-y-8">
        <PageBackButton />

        {detail.isError ? (
          <EmptyState
            emoji="Ò︵Ó"
            title="漫画源详情加载失败"
            actions={
              <Button type="button" variant="outline" size="sm" onClick={() => detail.refetch()}>
                重试
              </Button>
            }
          />
        ) : (
          <SourceComicDetail manga={detail.data} sourceId={sourceId} isUpdating={detail.isFetching} />
        )}
      </div>
      <BackTopButton />
    </main>
  )
}

function SourceComicDetail({
  manga,
  sourceId,
  isUpdating
}: {
  manga: SourceManga
  sourceId: string
  isUpdating: boolean
}) {
  const creators = [...(manga.authors ?? []), ...(manga.artists ?? [])]

  return (
    <div className="space-y-10">
      <section className="grid gap-6 md:grid-cols-[220px_minmax(0,1fr)] lg:grid-cols-[240px_minmax(0,1fr)] lg:gap-8">
        <ComicCover
          id={manga.key}
          title={manga.title}
          image={manga.cover ?? ''}
          loading="eager"
          className="mx-auto w-full max-w-60 md:max-w-none"
        />

        <div className="min-w-0 space-y-5 py-1">
          <div className="flex flex-wrap items-center gap-2">
            <Badge>{sourceId}</Badge>
            <Badge variant="outline">{formatStatus(manga.status)}</Badge>
            {isUpdating ? <span className="text-xs text-muted-foreground">正在更新详情…</span> : null}
          </div>

          <div className="space-y-2">
            <h1 className="text-2xl leading-tight font-bold tracking-normal sm:text-3xl lg:text-4xl">
              {manga.title}
            </h1>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <UserRoundIcon className="size-4" />
              <span>{creators.length > 0 ? creators.join(' / ') : '未知作者'}</span>
            </div>
          </div>

          <Separator />

          <p className="max-w-3xl whitespace-pre-line text-sm leading-7 text-muted-foreground">
            {manga.description || '暂无简介'}
          </p>

          {(manga.tags?.length ?? 0) > 0 ? (
            <div className="flex flex-wrap gap-2">
              {manga.tags?.map(tag => (
                <Badge key={tag} variant="secondary">
                  {tag}
                </Badge>
              ))}
            </div>
          ) : null}
        </div>
      </section>

      <SourceChapters chapters={manga.chapters ?? []} />
    </div>
  )
}

function SourceChapters({ chapters }: { chapters: SourceChapter[] }) {
  return (
    <section className="space-y-4">
      <div className="flex items-end justify-between gap-3">
        <div>
          <h2 className="text-xl font-semibold">章节</h2>
          <p className="text-sm text-muted-foreground">共 {chapters.length} 个章节</p>
        </div>
        <span className="text-xs text-muted-foreground">阅读能力将在后续接入</span>
      </div>

      {chapters.length === 0 ? (
        <p className="rounded-xl border border-dashed px-4 py-6 text-center text-sm text-muted-foreground">
          这个源没有返回章节
        </p>
      ) : (
        <div className="space-y-2">
          {chapters.map((chapter, index) => (
            <Card key={chapter.key} size="sm" className="py-0">
              <CardContent className="flex items-center justify-between gap-4 p-4">
                <div className="min-w-0">
                  <div className="truncate text-sm font-medium">
                    {chapter.title || formatChapterNumber(chapter, index)}
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {formatChapterNumber(chapter, index)}
                  </div>
                </div>
                <BookOpenIcon className="size-4 shrink-0 text-muted-foreground" />
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </section>
  )
}

function formatChapterNumber(chapter: SourceChapter, index: number) {
  if (chapter.chapterNumber != null) return `第 ${chapter.chapterNumber} 章`
  return `章节 ${index + 1}`
}

function formatStatus(status: string) {
  switch (status) {
    case 'ongoing':
      return '连载中'
    case 'completed':
      return '已完结'
    case 'cancelled':
      return '已取消'
    case 'hiatus':
      return '暂停连载'
    default:
      return '状态未知'
  }
}
