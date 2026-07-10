import { LoaderCircleIcon } from 'lucide-react'

import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle
} from '@/components/ui/drawer'
import type { ComicComment } from '@/lib/api/comic'
import { formatNumber } from '@/lib/format'
import { CommentSkeletonList } from './shared'
import { EmptyState } from '@/components/empty-state'
import { Button } from '@/components/ui/button'
import { useCallback } from 'react'

const CHINESE_DATE_FORMATTER = new Intl.DateTimeFormat('zh-CN', {
  year: 'numeric',
  month: 'long',
  day: 'numeric'
})

export type CommentsState = {
  isLoading: boolean
  isFetchingNextPage: boolean
  isError: boolean
  errorMessage?: string
  total: number
  comments: ComicComment[]
  hasNextPage: boolean
  onRetry: () => void
  onLoadMore: () => void
}

type CommentsDrawerProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  state: CommentsState
}

export function CommentsDrawer({ open, onOpenChange, state }: CommentsDrawerProps) {
  const { hasNextPage, isFetchingNextPage, onLoadMore } = state

  const handleScroll = useCallback(
    (event: React.UIEvent<HTMLDivElement>) => {
      if (!hasNextPage || isFetchingNextPage) {
        return
      }

      const element = event.currentTarget
      const distanceToBottom = element.scrollHeight - element.scrollTop - element.clientHeight

      if (distanceToBottom <= 80) {
        onLoadMore()
      }
    },
    [hasNextPage, isFetchingNextPage, onLoadMore]
  )

  return (
    <Drawer open={open} onOpenChange={onOpenChange} direction="right">
      <DrawerContent className="h-full w-[82vw] max-w-[320px] overflow-hidden rounded-l-2xl p-0 before:inset-0 before:rounded-l-2xl before:rounded-r-none data-[vaul-drawer-direction=right]:w-[82vw] data-[vaul-drawer-direction=right]:max-w-[320px] data-[vaul-drawer-direction=right]:sm:w-[420px] data-[vaul-drawer-direction=right]:sm:max-w-[420px]">
        <DrawerHeader className="border-b border-border/70 p-6">
          <DrawerTitle>评论</DrawerTitle>
          <DrawerDescription>共 {formatNumber(state.total)} 条评论</DrawerDescription>
        </DrawerHeader>

        <div className="min-h-0 flex-1 overflow-y-auto px-6 pb-6" onScroll={handleScroll}>
          {state.isLoading ? (
            <CommentSkeletonList />
          ) : state.isError ? (
            <EmptyState
              emoji="Ò︵Ó"
              title="数据加载失败"
              actions={
                <Button type="button" variant="outline" size="sm" onClick={state.onRetry}>
                  重试
                </Button>
              }
            />
          ) : state.comments.length === 0 ? (
            <EmptyState emoji="(･o･;)" title="暂无评论" />
          ) : (
            <div className="space-y-5">
              {state.comments.map(comment => (
                <CommentItem key={comment.id} comment={comment} />
              ))}
              <CommentsEndState state={state} />
            </div>
          )}
        </div>
      </DrawerContent>
    </Drawer>
  )
}

function CommentsEndState({ state }: { state: CommentsState }) {
  if (state.isFetchingNextPage) {
    return (
      <div className="flex items-center justify-center gap-2 py-4 text-xs text-muted-foreground">
        <LoaderCircleIcon className="size-3.5 animate-spin" />
        正在加载评论
      </div>
    )
  }

  return (
    <p className="py-2 text-center text-xs text-muted-foreground">
      {state.hasNextPage ? '继续向下滚动加载更多' : '暂无更多评论'}
    </p>
  )
}

function CommentItem({ comment }: { comment: ComicComment }) {
  const name = getCommentAuthorName(comment)
  const content = getCommentText(comment, '这条评论没有内容')

  return (
    <div className="space-y-3 px-px py-1">
      <div className="space-y-1">
        <div className="truncate text-sm font-medium">{name}</div>
        <div className="text-xs text-muted-foreground">{formatCommentTime(comment.time)}</div>
      </div>

      <p className="text-xs text-card-foreground">{content}</p>

      {comment.replies.length > 0 ? (
        <div className="space-y-2 rounded-md bg-muted/60 p-3">
          {comment.replies.map(reply => (
            <ReplyItem key={reply.id} reply={reply} />
          ))}
        </div>
      ) : null}
    </div>
  )
}

function ReplyItem({ reply }: { reply: ComicComment }) {
  const name = getCommentAuthorName(reply)
  const content = getCommentText(reply, '这条回复没有内容')

  return (
    <div className="text-xs">
      <span className="font-medium">{name}</span>
      <span className="text-muted-foreground"> ：{content}</span>
    </div>
  )
}

function getCommentAuthorName(comment: ComicComment) {
  return comment.nickname || comment.username || `用户 ${comment.userId}`
}

function getCommentText(comment: ComicComment, emptyText: string) {
  return htmlToText(comment.content) || emptyText
}

function formatCommentTime(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? '' : CHINESE_DATE_FORMATTER.format(date)
}

function htmlToText(value: string) {
  const { body } = new DOMParser().parseFromString(value, 'text/html')
  return body.textContent?.trim() ?? ''
}
