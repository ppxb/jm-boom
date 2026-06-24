import { useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { ImageIcon, SearchIcon } from 'lucide-react'
import type { FormEvent, ReactNode } from 'react'
import { useState } from 'react'

import { Button } from '@/components/ui/button'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput
} from '@/components/ui/input-group'
import { searchAlbums } from '@/lib/api/search'

export const Route = createFileRoute('/_app/')({
  component: HomePage
})

function HomePage() {
  const [inputValue, setInputValue] = useState('')
  const [searchQuery, setSearchQuery] = useState('')

  const search = useQuery({
    queryKey: ['jm-search', 2, searchQuery],
    queryFn: () => searchAlbums({ query: searchQuery }),
    enabled: searchQuery.length > 0
  })

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setSearchQuery(inputValue.trim())
  }

  return (
    <main className="mx-auto flex min-h-full w-full max-w-5xl flex-col px-6 py-10 sm:px-12">
      <form onSubmit={handleSubmit} className="mx-auto mt-[18vh] w-full max-w-xl">
        <InputGroup className="h-10 bg-background">
          <InputGroupAddon align="inline-start">
            <SearchIcon className="text-muted-foreground" />
          </InputGroupAddon>
          <InputGroupInput
            value={inputValue}
            onChange={event => setInputValue(event.currentTarget.value)}
            placeholder="Search keyword or JMID"
          />
          <InputGroupAddon align="inline-end">
            <InputGroupButton type="submit" disabled={inputValue.trim().length === 0}>
              Search
            </InputGroupButton>
          </InputGroupAddon>
        </InputGroup>
      </form>

      <section className="mt-10 pb-24 sm:pb-10">
        {search.isFetching ? (
          <SearchState label="Searching..." />
        ) : search.isError ? (
          <SearchState
            label="Search failed"
            description={search.error.message}
            action={
              <Button variant="outline" size="sm" onClick={() => search.refetch()}>
                Retry
              </Button>
            }
          />
        ) : search.data != null && search.data.items.length === 0 ? (
          <SearchState label="No results" description={`No albums found for "${search.data.query}".`} />
        ) : search.data != null ? (
          <div className="space-y-4">
            <div className="flex items-center justify-between gap-4 text-sm text-muted-foreground">
              <span>{search.data.total} results</span>
              <span>
                {search.data.redirectAid ? `Direct match JM${search.data.redirectAid}` : `Page ${search.data.page}`}
              </span>
            </div>
            <div className="grid gap-3">
              {search.data.items.map(item => (
                <article
                  key={item.id}
                  className="overflow-hidden rounded-xl border bg-card text-card-foreground shadow-sm"
                >
                  <div className="flex gap-4 p-4">
                    <ComicCover title={item.title} image={item.image} />

                    <div className="min-w-0 flex-1">
                      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                        <div className="min-w-0 space-y-2">
                          <div className="text-xs font-medium text-muted-foreground">JM{item.id}</div>
                          <h2 className="line-clamp-2 text-base font-medium">{item.title}</h2>
                          {item.isRedirect ? (
                            <p className="text-sm text-muted-foreground">Direct album match</p>
                          ) : null}
                          {item.author ? (
                            <p className="line-clamp-1 text-sm text-muted-foreground">{item.author}</p>
                          ) : null}
                        </div>
                      </div>

                      {item.tags.length > 0 ? (
                        <div className="mt-3 flex flex-wrap gap-1.5">
                          {item.tags.slice(0, 8).map(tag => (
                            <span
                              key={tag}
                              className="rounded-md bg-muted px-2 py-1 text-xs text-muted-foreground"
                            >
                              {tag}
                            </span>
                          ))}
                        </div>
                      ) : null}
                    </div>
                  </div>
                </article>
              ))}
            </div>
          </div>
        ) : null}
      </section>
    </main>
  )
}

function ComicCover({ title, image }: { title: string; image: string }) {
  const [hasError, setHasError] = useState(false)

  return (
    <div className="flex aspect-[3/4] w-24 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted sm:w-28">
      {image && !hasError ? (
        <img
          src={image}
          alt={title}
          loading="lazy"
          referrerPolicy="no-referrer"
          className="h-full w-full object-cover"
          onError={() => setHasError(true)}
        />
      ) : (
        <div className="flex flex-col items-center gap-2 px-3 text-center text-xs text-muted-foreground">
          <ImageIcon className="size-5" />
          <span>No cover</span>
        </div>
      )}
    </div>
  )
}

function SearchState({
  label,
  description,
  action
}: {
  label: string
  description?: string
  action?: ReactNode
}) {
  return (
    <div className="mx-auto flex max-w-xl flex-col items-center gap-3 rounded-lg border border-dashed bg-card/70 px-6 py-8 text-center">
      <p className="text-sm font-medium">{label}</p>
      {description ? <p className="text-sm text-muted-foreground">{description}</p> : null}
      {action}
    </div>
  )
}
