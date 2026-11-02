import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { act, useState, type ReactNode } from 'react'
import { createRoot, type Root } from 'react-dom/client'

import { ConfirmDialog } from './confirm-dialog'
import { FilterSelect } from './filter-select'
import { SideDrawerContent } from './side-drawer-content'
import { Button } from './ui/button'
import { Drawer, DrawerClose, DrawerDescription, DrawerTitle, DrawerTrigger } from './ui/drawer'
import { ReaderProgressSlider } from '@/features/reader/reader-progress-slider'
import { ReaderSettingsMenu } from '@/features/reader/reader-settings-menu'

let root: Root

beforeEach(() => {
  vi.stubGlobal('IS_REACT_ACT_ENVIRONMENT', true)
  const container = document.createElement('div')
  document.body.append(container)
  root = createRoot(container)
})

afterEach(async () => {
  await act(async () => root.unmount())
  document.body.replaceChildren()
  vi.unstubAllGlobals()
})

async function render(children: ReactNode) {
  await act(async () => root.render(children))
}

async function click(selector: string) {
  const element = document.querySelector<HTMLElement>(selector)
  expect(element).not.toBeNull()
  await act(async () => element!.click())
}

test('a button rendered as a link keeps its destination without a nested button', async () => {
  await render(<Button nativeButton={false} render={<a href="/explore/search">搜索</a>} />)
  expect(document.querySelector('a')?.getAttribute('href')).toBe('/explore/search')
  expect(document.querySelector('a')?.textContent).toBe('搜索')
  expect(document.querySelector('button')).toBeNull()
})

test('filter choices show their label before opening and submit the selected value', async () => {
  const changed = vi.fn()
  function Filter() {
    const [value, setValue] = useState('first')
    return (
      <FilterSelect
        value={value}
        placeholder="排序"
        options={[
          { value: 'first', label: '从新到旧' },
          { value: 'second', label: '最多观看' }
        ]}
        onValueChange={next => {
          setValue(next)
          changed(next)
        }}
      />
    )
  }
  await render(<Filter />)
  expect(document.querySelector('[role="combobox"]')?.textContent).toContain('从新到旧')
  await click('[role="combobox"]')
  const options = Array.from(document.querySelectorAll<HTMLElement>('[role="option"]'))
  expect(options).toHaveLength(2)
  await act(async () => options[1].click())
  expect(changed).toHaveBeenCalledWith('second')
  expect(document.querySelector('[role="combobox"]')?.textContent).toContain('最多观看')
})

test('confirmation submits once and closes the dialog', async () => {
  const confirm = vi.fn()
  await render(
    <ConfirmDialog
      trigger={<Button>删除任务</Button>}
      title="确认删除"
      description="删除选中的任务"
      onConfirm={confirm}
    />
  )
  await click('[data-slot="alert-dialog-trigger"]')
  expect(document.querySelector('[role="alertdialog"]')).not.toBeNull()
  await click('[data-slot="alert-dialog-action"]')
  expect(confirm).toHaveBeenCalledTimes(1)
  expect(
    document.querySelector('[data-slot="alert-dialog-trigger"]')?.getAttribute('aria-expanded')
  ).toBe('false')
})

test('side drawers open with their title and close through the composed button', async () => {
  await render(
    <Drawer swipeDirection="right">
      <DrawerTrigger render={<Button>章节</Button>} />
      <SideDrawerContent>
        <DrawerTitle>章节目录</DrawerTitle>
        <DrawerDescription>选择章节</DrawerDescription>
        <DrawerClose render={<Button>关闭</Button>} />
      </SideDrawerContent>
    </Drawer>
  )
  await click('[data-slot="drawer-trigger"]')
  expect(document.querySelector('[role="dialog"]')?.textContent).toContain('章节目录')
  expect(
    document.querySelector('[data-slot="drawer-popup"]')?.getAttribute('data-swipe-direction')
  ).toBe('right')
  await click('[data-slot="drawer-close"]')
  expect(
    document.querySelector('[data-slot="drawer-trigger"]')?.getAttribute('aria-expanded')
  ).toBe('false')
})

test('the reader menu opens and exposes its radio choices', async () => {
  await render(<ReaderSettingsMenu />)
  await click('[data-slot="dropdown-menu-trigger"]')
  expect(document.querySelector('[role="menu"]')?.textContent).toContain('阅读模式')
  expect(document.querySelectorAll('[role="menuitemradio"]')).toHaveLength(2)
})

test('the reader slider labels its input and handles a single-page chapter', async () => {
  await render(<ReaderProgressSlider currentIndex={0} pageCount={1} onPageChange={vi.fn()} />)
  const input = document.querySelector<HTMLInputElement>('input[type="range"]')!
  expect(input.disabled).toBe(true)
  expect(input.getAttribute('aria-label')).toBe('阅读进度')
  expect(input.getAttribute('aria-valuetext')).toBe('第 1 张，共 1 张')
  expect(input.value).toBe('0')
})
