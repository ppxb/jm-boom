import { Button } from '@/components/ui/button'

export function StatePanel({
  title,
  description,
  onRetry
}: {
  title: string
  description?: string
  onRetry?: () => void
}) {
  return (
    <div className="flex min-h-40 flex-col items-center justify-center gap-3 rounded-md border border-dashed border-border/70 bg-card/60 px-6 py-8 text-center">
      <p className="text-sm font-medium">{title}</p>
      {description ? <p className="max-w-md text-sm text-muted-foreground">{description}</p> : null}
      {onRetry ? (
        <Button type="button" variant="outline" size="sm" onClick={onRetry}>
          重试
        </Button>
      ) : null}
    </div>
  )
}
