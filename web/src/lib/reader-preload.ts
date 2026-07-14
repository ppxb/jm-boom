import { preloadComicReadPage } from './api/reader'
import { READER } from './constants'

const COMPLETED_PATH_LIMIT = 1000

type PreloadJob = {
  controller: AbortController
  owners: Set<string>
}

class ReaderPreloadScheduler {
  private readonly active = new Map<string, PreloadJob>()
  private readonly completed = new Set<string>()
  private readonly completedOrder: string[] = []
  private readonly queue: string[] = []
  private readonly queued = new Map<string, Set<string>>()
  private readonly scopes = new Map<string, Set<string>>()

  setScope(scope: string, paths: string[]) {
    const desiredPaths = new Set(paths.filter(Boolean))
    const previousPaths = this.scopes.get(scope) ?? new Set<string>()

    for (const path of previousPaths) {
      if (!desiredPaths.has(path)) {
        this.removeOwner(scope, path)
      }
    }

    this.scopes.set(scope, desiredPaths)

    for (const path of desiredPaths) {
      this.addOwner(scope, path)
    }

    this.pump()
  }

  clearScope(scope: string) {
    const paths = this.scopes.get(scope)

    if (!paths) {
      return
    }

    this.scopes.delete(scope)

    for (const path of paths) {
      this.removeOwner(scope, path)
    }

    this.pump()
  }

  private addOwner(scope: string, path: string) {
    if (this.completed.has(path)) {
      return
    }

    const activeJob = this.active.get(path)

    if (activeJob) {
      if (!activeJob.controller.signal.aborted) {
        activeJob.owners.add(scope)
        return
      }

      const queuedOwners = this.queued.get(path)

      if (queuedOwners) {
        queuedOwners.add(scope)
      } else {
        this.queued.set(path, new Set([scope]))
        this.queue.push(path)
      }
      return
    }

    const queuedOwners = this.queued.get(path)

    if (queuedOwners) {
      queuedOwners.add(scope)
      return
    }

    this.queued.set(path, new Set([scope]))
    this.queue.push(path)
  }

  private removeOwner(scope: string, path: string) {
    const queuedOwners = this.queued.get(path)

    if (queuedOwners) {
      queuedOwners.delete(scope)

      if (queuedOwners.size === 0) {
        this.queued.delete(path)
      }
    }

    const activeJob = this.active.get(path)

    if (!activeJob) {
      return
    }

    activeJob.owners.delete(scope)

    if (activeJob.owners.size === 0) {
      activeJob.controller.abort()
    }
  }

  private pump() {
    while (this.active.size < READER.PREFETCH_CONCURRENCY) {
      const path = this.nextQueuedPath()

      if (!path) {
        return
      }

      const owners = this.queued.get(path)
      this.queued.delete(path)

      if (!owners || owners.size === 0 || this.completed.has(path)) {
        continue
      }

      const controller = new AbortController()
      const job = { controller, owners }
      this.active.set(path, job)

      void preloadComicReadPage(path, controller.signal)
        .then(() => this.markCompleted(path))
        .catch(error => {
          if (!controller.signal.aborted && import.meta.env.DEV) {
            console.debug('Reader page preload failed', { path, error })
          }
        })
        .finally(() => {
          if (this.active.get(path) === job) {
            this.active.delete(path)
          }

          this.pump()
        })
    }
  }

  private nextQueuedPath() {
    while (this.queue.length > 0) {
      const path = this.queue.shift()

      if (path && this.queued.has(path)) {
        return path
      }
    }

    return undefined
  }

  private markCompleted(path: string) {
    if (this.completed.has(path)) {
      return
    }

    this.completed.add(path)
    this.completedOrder.push(path)

    while (this.completedOrder.length > COMPLETED_PATH_LIMIT) {
      const expiredPath = this.completedOrder.shift()

      if (expiredPath) {
        this.completed.delete(expiredPath)
      }
    }
  }
}

const scheduler = new ReaderPreloadScheduler()

export function setReaderPreloadScope(scope: string, paths: string[]) {
  scheduler.setScope(scope, paths)
}

export function clearReaderPreloadScope(scope: string) {
  scheduler.clearScope(scope)
}
