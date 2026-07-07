export function DownloadEmptyState({ label }: { label: string }) {
  return (
    <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-4 text-center">
      <p className="text-6xl font-bold text-foreground">(˘･_･˘)</p>
      <p className="text-sm text-muted-foreground">{label}</p>
    </div>
  )
}
