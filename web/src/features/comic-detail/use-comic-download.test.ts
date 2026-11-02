import { expect, test } from 'vitest'
import { toDownloadChapterOptions } from './use-comic-download'

test('download order follows chapter numbers regardless of the displayed order', () => {
  const source = [
    { id: '102', title: '第二章', sort: '2' },
    { id: '101', title: '第一章', sort: '1' }
  ]
  const descending = toDownloadChapterOptions('100', source, source)
  const ascending = toDownloadChapterOptions('100', [...source].reverse(), source)
  expect(descending.map(chapter => chapter.order)).toEqual([2, 1])
  expect(ascending.map(chapter => chapter.order)).toEqual([1, 2])
  expect([...descending].sort((left, right) => left.order - right.order)).toEqual(ascending)
})

test('chapters without numbers keep upstream order and a standalone comic has one chapter', () => {
  const source = [
    { id: '203', title: '序章', sort: '' },
    { id: '202', title: '终章', sort: '' }
  ]
  expect(
    toDownloadChapterOptions('200', [...source].reverse(), source).map(chapter => chapter.order)
  ).toEqual([2, 1])
  expect(toDownloadChapterOptions('300', [], [])).toEqual([
    { chapterId: '300', title: '单章', order: 1 }
  ])
})
