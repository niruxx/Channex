import { useCallback, useEffect, useRef, useState } from 'react'
import type { Post } from '../types'
import { useSitesStore } from '../store/useSitesStore'
import { getAdapter } from '../lib/adapters'

const cache = new Map<string, Post[]>()

export function useCatalog(siteId: string, boardCode: string | null) {
  const site = useSitesStore((s) => s.getSite(siteId))
  const key = `${siteId}/${boardCode}`
  const [threads, setThreads] = useState<Post[] | null>(() => (boardCode ? cache.get(key) ?? null : null))
  const [loading, setLoading] = useState(!threads)
  const [error, setError] = useState<string | null>(null)
  const seq = useRef(0)

  const load = useCallback(async () => {
    if (!site || !boardCode) return
    const mySeq = ++seq.current
    setLoading(true)
    setError(null)
    try {
      const adapter = getAdapter(site)
      const data = await adapter.fetchCatalog(site, boardCode)
      data.sort((a, b) => {
        if (!!a.sticky !== !!b.sticky) return a.sticky ? -1 : 1
        return b.timestamp - a.timestamp
      })
      if (mySeq === seq.current) {
        cache.set(key, data)
        setThreads(data)
      }
    } catch (e) {
      if (mySeq === seq.current) setError(e instanceof Error ? e.message : String(e))
    } finally {
      if (mySeq === seq.current) setLoading(false)
    }
  }, [site, boardCode, key])

  useEffect(() => {
    setThreads(boardCode ? cache.get(key) ?? null : null)
    load()
  }, [key, load, boardCode])

  return { threads, loading, error, reload: load }
}
