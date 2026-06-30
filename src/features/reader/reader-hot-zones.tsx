export function ReaderHotZones({
  onPrevious,
  onNext
}: {
  onPrevious: () => void
  onNext: () => void
}) {
  return (
    <>
      <button
        type="button"
        aria-label="上一页"
        className="absolute top-20 bottom-20 left-0 z-10 w-1/5 cursor-pointer"
        onClick={event => {
          event.stopPropagation()
          onPrevious()
        }}
      />
      <button
        type="button"
        aria-label="下一页"
        className="absolute top-20 right-0 bottom-20 z-10 w-1/5 cursor-pointer"
        onClick={event => {
          event.stopPropagation()
          onNext()
        }}
      />
    </>
  )
}
