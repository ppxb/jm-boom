import { RefreshCwIcon } from 'lucide-react'

import { PageHeader } from '@/components/page-header'
import { Button } from '@/components/ui/button'

export function FeedHeader({
  title,
  description,
  isFetching,
  onRefresh
}: {
  title: string
  description: string
  isFetching?: boolean
  onRefresh?: () => void
}) {
  return (
    <PageHeader title={title} desc={description}>
      {onRefresh ? (
        <Button
          type="button"
          variant="outline"
          size="icon"
          disabled={isFetching}
          onClick={onRefresh}
          aria-label="刷新"
          className="cursor-pointer"
        >
          <RefreshCwIcon className={isFetching ? 'animate-spin' : undefined} />
        </Button>
      ) : null}
    </PageHeader>
  )
}
