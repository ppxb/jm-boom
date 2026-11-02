import type { ReaderPageItem } from './types'

export function readerSpreadStarts(pages: ReaderPageItem[], doublePage: boolean) {
  const starts: number[] = []
  for (let index = 0; index < pages.length;) {
    starts.push(index)
    const current = pages[index]
    const next = pages[index + 1]
    const canPair =
      doublePage &&
      next &&
      current.chapterId === next.chapterId &&
      !isSpread(current) &&
      !isSpread(next)
    index += canPair ? 2 : 1
  }
  return starts
}

export function readerSpread(pages: ReaderPageItem[], index: number, doublePage: boolean) {
  const starts = readerSpreadStarts(pages, doublePage)
  let slot = 0
  while (slot + 1 < starts.length && starts[slot + 1] <= index) slot++
  const start = starts[slot] ?? 0
  const end = starts[slot + 1] ?? pages.length
  return { start, end, previous: starts[slot - 1] ?? -1 }
}

function isSpread(page: ReaderPageItem) {
  return Boolean(page.width && page.height && page.width > page.height)
}
