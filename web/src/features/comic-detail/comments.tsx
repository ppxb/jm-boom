import { useInfiniteQuery } from '@tanstack/react-query'
import { LoaderCircleIcon } from 'lucide-react'
import { memo, useCallback, useMemo, type UIEvent } from 'react'

import { EmptyState } from '@/components/empty-state'
import { SideDrawerContent } from '@/components/side-drawer-content'
import { Button } from '@/components/ui/button'
import { Drawer, DrawerDescription, DrawerHeader, DrawerTitle } from '@/components/ui/drawer'
import { getComicComments, type ComicComment } from '@/lib/api/comic'
import { CACHE } from '@/lib/constants'
import { formatNumber } from '@/lib/format'
import { queryKeys } from '@/lib/query-keys'
import { CommentSkeletonList } from './shared'

const CHINESE_DATE_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  year: 'numeric',
  month: 'long',
  day: 'numeric'
})

type CommentsDrawerProps = {
  comicId: string
  total: number
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CommentsDrawer({ comicId, total, open, onOpenChange }: CommentsDrawerProps) {
  const commentsQuery = useInfiniteQuery({
    queryKey: queryKeys.comicComments(comicId),
    queryFn: ({ pageParam }) => getComicComments({ comicId, page: pageParam }),
    initialPageParam: 1,
    enabled: open,
    staleTime: CACHE.COMMENTS_STALE_TIME,
    gcTime: CACHE.COMMENTS_GC_TIME,
    refetchOnMount: false,
    refetchOnWindowFocus: false,
    getNextPageParam: (lastPage, allPages) => {
      const loadedCount = allPages.reduce((sum, page) => sum + page.comments.length, 0)

      if (lastPage.comments.length === 0 || loadedCount >= lastPage.total) {
        return undefined
      }

      return lastPage.page + 1
    }
  })
  const comments = useMemo(
    () => commentsQuery.data?.pages.flatMap(page => page.comments) ?? [],
    [commentsQuery.data]
  )
  const {
    fetchNextPage,
    hasNextPage,
    isError,
    isFetchingNextPage,
    isLoading,
    refetch
  } = commentsQuery
  const commentTotal = commentsQuery.data?.pages[0]?.total ?? total

  const handleScroll = useCallback(
    (event: UIEvent<HTMLDivElement>) => {
      if (!hasNextPage || isFetchingNextPage) {
        return
      }

      const element = event.currentTarget
      const distanceToBottom = element.scrollHeight - element.scrollTop - element.clientHeight

      if (distanceToBottom <= 80) {
        void fetchNextPage({ cancelRefetch: false })
      }
    },
    [fetchNextPage, hasNextPage, isFetchingNextPage]
  )

  return (
    <Drawer open={open} onOpenChange={onOpenChange} direction="right">
      <SideDrawerContent>
        <DrawerHeader className="border-b border-border/70 p-6">
          <DrawerTitle>评论</DrawerTitle>
          <DrawerDescription>共 {formatNumber(commentTotal)} 条评论</DrawerDescription>
        </DrawerHeader>

        <div
          className="min-h-0 flex-1 scroll-fade-y overflow-y-auto px-6 pt-2 pb-6"
          onScroll={handleScroll}
        >
          {isLoading ? (
            <CommentSkeletonList />
          ) : isError ? (
            <EmptyState
              emoji="Ò︵Ó"
              title="数据加载失败"
              actions={
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => refetch()}
                >
                  重试
                </Button>
              }
            />
          ) : comments.length === 0 ? (
            <EmptyState emoji="(･o･;)" title="暂无评论" />
          ) : (
            <div className="space-y-5">
              {comments.map(comment => (
                <CommentItem key={comment.id} comment={comment} />
              ))}
              <CommentsEndState
                isFetchingNextPage={isFetchingNextPage}
                hasNextPage={hasNextPage}
              />
            </div>
          )}
        </div>
      </SideDrawerContent>
    </Drawer>
  )
}

function CommentsEndState({
  isFetchingNextPage,
  hasNextPage
}: {
  isFetchingNextPage: boolean
  hasNextPage: boolean
}) {
  if (isFetchingNextPage) {
    return (
      <div className="flex items-center justify-center gap-2 py-4 text-xs text-muted-foreground">
        <LoaderCircleIcon className="size-3.5 animate-spin" />
        正在加载评论
      </div>
    )
  }

  return (
    <p className="py-2 text-center text-xs text-muted-foreground">
      {hasNextPage ? '继续向下滚动加载更多' : '暂无更多评论'}
    </p>
  )
}

const CommentItem = memo(function CommentItem({ comment }: { comment: ComicComment }) {
  const name = getCommentAuthorName(comment)

  return (
    <div className="space-y-3 px-px py-1">
      <div className="space-y-1">
        <div className="truncate text-sm font-medium">{name}</div>
        <div className="text-xs text-muted-foreground">{formatCommentTime(comment.time)}</div>
      </div>

      <p className="text-xs text-card-foreground">{comment.content}</p>

      {comment.replies.length > 0 ? (
        <div className="space-y-2 rounded-md bg-muted/60 p-3">
          {comment.replies.map(reply => (
            <ReplyItem key={reply.id} reply={reply} />
          ))}
        </div>
      ) : null}
    </div>
  )
})

const ReplyItem = memo(function ReplyItem({ reply }: { reply: ComicComment }) {
  const name = getCommentAuthorName(reply)

  return (
    <div className="text-xs">
      <span className="font-medium">{name}</span>
      <span className="text-muted-foreground"> ：{reply.content}</span>
    </div>
  )
})

function getCommentAuthorName(comment: ComicComment) {
  return comment.nickname || comment.username || '用户 ' + comment.userId
}

function formatCommentTime(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? '' : CHINESE_DATE_FORMATTER.format(date)
}
