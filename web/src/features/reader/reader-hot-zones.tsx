import type { ReaderPageDirection } from '@/stores/settings-store'

export function ReaderHotZones({
  pageDirection,
  onPrevious,
  onNext
}: {
  pageDirection: ReaderPageDirection
  onPrevious: () => void
  onNext: () => void
}) {
  const leftAction = pageDirection === 'rtl' ? onNext : onPrevious
  const rightAction = pageDirection === 'rtl' ? onPrevious : onNext

  return (
    <>
      <button
        type="button"
        aria-label={pageDirection === 'rtl' ? '下一页' : '上一页'}
        className="absolute top-20 bottom-20 left-0 z-20 w-[12vw] min-w-20 max-w-36 cursor-pointer border-0 bg-transparent p-0"
        onClick={event => {
          event.stopPropagation()
          leftAction()
        }}
      />
      <button
        type="button"
        aria-label={pageDirection === 'rtl' ? '上一页' : '下一页'}
        className="absolute top-20 right-0 bottom-20 z-20 w-[12vw] min-w-20 max-w-36 cursor-pointer border-0 bg-transparent p-0"
        onClick={event => {
          event.stopPropagation()
          rightAction()
        }}
      />
    </>
  )
}
