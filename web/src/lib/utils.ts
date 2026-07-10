import { clsx, type ClassValue } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

export function currentChinaWeekday() {
  const date = new Date()
  const chinaDate = new Date(date.getTime() + (date.getTimezoneOffset() + 480) * 60 * 1000)
  const day = chinaDate.getDay()

  return day === 0 ? 7 : day
}

export function parseStringSearch(value: unknown, fallback = '') {
  return typeof value === 'string' ? value : fallback
}

export function parsePositivePage(value: unknown) {
  const page = Number.parseInt(String(value ?? ''), 10)

  return Number.isFinite(page) && page > 0 ? page : 1
}
