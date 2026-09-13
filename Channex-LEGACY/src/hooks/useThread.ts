import { useCallback, useEffect, useRef, useState } from 'react'
import type { ThreadData } from '../types'
import { useSitesStore } from '../store/useSitesStore'
import { getAdapter } from '../lib/adapters'

const cache = new Map<string, ThreadData>()

export function useThread(siteId: string, boardCode: string | null, threadId: string | null) {
  const site = useSitesStore((s) => s.getSite(siteId))
  const key = `${siteId}/${boardCode}/${threadId}`
  const [data, setData] = useState<ThreadData | null>(() => (threadId ? cache.get(key) ?? null : null))
  const [loading, setLoading] = useState(!data)
  const [error, setError] = useState<string | null>(null)
  const seq = useRef(0)

  const load = useCallback(async () => {
    if (!site || !boardCode || !threadId) return
    const mySeq = ++seq.current
    setLoading(true)
    setError(null)
    try {
      const adapter = getAdapter(site)
      const result = await adapter.fetchThread(site, boardCode, threadId)
      if (mySeq === seq.current) {
        cache.set(key, result)
        setData(result)
      }
    } catch (e) {
      if (mySeq === seq.current) setError(e instanceof Error ? e.message : String(e))
    } finally {
      if (mySeq === seq.current) setLoading(false)
    }
  }, [site, boardCode, threadId, key])

  useEffect(() => {
    setData(threadId ? cache.get(key) ?? null : null)
    load()
  }, [key, load, threadId])

  return { data, loading, error, reload: load }
}
