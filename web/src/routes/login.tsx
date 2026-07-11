import { createFileRoute } from '@tanstack/react-router'
import { LoaderCircleIcon } from 'lucide-react'
import { useEffect, useState, type FormEvent } from 'react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { grantAccessGate, hasAccessGateGrant, loadAccessGateConfig } from '@/lib/access-gate'
import { loginAccessGate } from '@/lib/api/auth'

type LoginSearch = {
  redirect?: string
}

export const Route = createFileRoute('/login')({
  validateSearch: (search: Record<string, unknown>): LoginSearch => ({
    redirect: typeof search.redirect === 'string' ? search.redirect : undefined
  }),
  component: LoginPage
})

function LoginPage() {
  const { redirect } = Route.useSearch()
  const [password, setPassword] = useState('')
  const [checking, setChecking] = useState(true)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState('')

  useEffect(() => {
    let active = true
    void loadAccessGateConfig()
      .then(config => {
        if (!active) return
        if (!config.enabled || hasAccessGateGrant()) {
          window.location.replace(safeRedirect(redirect))
          return
        }
        setChecking(false)
      })
      .catch(requestError => {
        if (!active) return
        setError(requestError instanceof Error ? requestError.message : String(requestError))
        setChecking(false)
      })
    return () => {
      active = false
    }
  }, [redirect])

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!password || submitting) return
    setSubmitting(true)
    setError('')
    try {
      const result = await loginAccessGate(password)
      if (!result.success) throw new Error('访问密码错误')
      grantAccessGate()
      window.location.replace(safeRedirect(redirect))
    } catch (requestError) {
      setError(requestError instanceof Error ? requestError.message : String(requestError))
      setSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-dvh items-center justify-center bg-muted px-4 py-10">
      <Card className="w-full max-w-sm">
        <CardHeader className="items-center text-center">
          <CardTitle>访问 JM Boom</CardTitle>
          <CardDescription>请输入部署时配置的访问密码</CardDescription>
        </CardHeader>
        <CardContent>
          {checking ? (
            <div className="flex items-center justify-center py-6 text-sm text-muted-foreground">
              <LoaderCircleIcon className="mr-2 size-4 animate-spin" />
              正在检查访问配置
            </div>
          ) : (
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <Label htmlFor="access-password">访问密码</Label>
                <Input
                  id="access-password"
                  type="password"
                  value={password}
                  autoComplete="current-password"
                  autoFocus
                  disabled={submitting}
                  onChange={event => setPassword(event.target.value)}
                />
              </div>
              {error ? <p className="text-sm text-destructive">{error}</p> : null}
              <Button type="submit" className="w-full" disabled={!password || submitting}>
                {submitting ? <LoaderCircleIcon className="size-4 animate-spin" /> : null}
                进入
              </Button>
            </form>
          )}
        </CardContent>
      </Card>
    </main>
  )
}

function safeRedirect(redirect: string | undefined) {
  return redirect?.startsWith('/') && !redirect.startsWith('//') && !redirect.startsWith('/login')
    ? redirect
    : '/bookshelf'
}
