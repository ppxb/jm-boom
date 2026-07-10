import React from 'react'
import ReactDOM from 'react-dom/client'
import { createRouter, RouterProvider, stringifySearchWith } from '@tanstack/react-router'

import { Providers } from '@/components/providers'

import { routeTree } from './routeTree.gen'
import './styles/globals.css'

export const router = createRouter({
  routeTree,
  parseSearch: parseSearchParams,
  stringifySearch: stringifySearchWith(JSON.stringify),
  defaultPreload: 'intent',
  defaultPreloadStaleTime: 5000,
  scrollRestoration: true
})

function parseSearchParams(searchString: string) {
  const search = new URLSearchParams(
    searchString.startsWith('?') ? searchString.slice(1) : searchString
  )
  const result: Record<string, unknown> = {}

  for (const [key, rawValue] of search.entries()) {
    const value = parseSearchValue(rawValue)
    const previousValue = result[key]

    if (previousValue === undefined) {
      result[key] = value
    } else if (Array.isArray(previousValue)) {
      previousValue.push(value)
    } else {
      result[key] = [previousValue, value]
    }
  }

  return result
}

function parseSearchValue(value: string) {
  if (value.startsWith('"') || value.startsWith('{') || value.startsWith('[')) {
    try {
      return JSON.parse(value)
    } catch {
      return value
    }
  }

  return value
}

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}

const root = document.getElementById('root') as HTMLElement

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <Providers>
      <RouterProvider router={router} />
    </Providers>
  </React.StrictMode>
)
