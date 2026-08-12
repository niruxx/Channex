import { useEffect, useMemo, useState } from 'react'
import { motion } from 'motion/react'
import { Search, RefreshCw, Plus } from 'lucide-react'
import { useSitesStore } from '../../store/useSitesStore'
import { useNavStore } from '../../store/useNavStore'
import { cn } from '../../lib/cn'

export function BoardList() {
  const siteId = useNavStore((s) => s.siteId)
  const currentBoard = useNavStore((s) => s.boardCode)
  const goCatalog = useNavStore((s) => s.goCatalog)
  const boards = useSitesStore((s) => s.boards[siteId])
  const loading = useSitesStore((s) => s.boardsLoading[siteId])
  const error = useSitesStore((s) => s.boardsError[siteId])
  const loadBoards = useSitesStore((s) => s.loadBoards)
  const addManualBoard = useSitesStore((s) => s.addManualBoard)

  const [query, setQuery] = useState('')
  const [manualCode, setManualCode] = useState('')

  useEffect(() => {
    loadBoards(siteId)
  }, [siteId, loadBoards])

  const filtered = useMemo(() => {
    const list = boards ?? []
    if (!query.trim()) return list
    const q = query.trim().toLowerCase()
    return list.filter((b) => b.code.toLowerCase().includes(q) || b.title.toLowerCase().includes(q))
  }, [boards, query])

  return (
    <div className="card flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-1.5 p-2.5">
        <div className="relative flex-1">
          <Search size={13} className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-faint" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Filter boards"
            className="w-full rounded-lg border border-transparent bg-surface-3 py-1.5 pl-8 pr-2 text-xs text-ink outline-none transition-colors focus:border-accent/50"
          />
        </div>
        <button
          type="button"
          onClick={() => loadBoards(siteId, true)}
          title="Refresh board list"
          className="btn-icon size-7 rounded-lg bg-surface-3"
        >
          <RefreshCw size={13} className={cn(loading && 'animate-spin')} />
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-2">
        {filtered.length === 0 && !loading && (
          <p className="px-2.5 py-4 text-xs leading-relaxed text-ink-faint">
            {error ? 'Could not auto-discover boards for this site.' : 'No boards yet.'} Add one by its board code below.
          </p>
        )}
        <ul className="space-y-0.5">
          {filtered.map((b) => {
            const active = b.code === currentBoard
            return (
              <motion.li key={b.code} whileHover={{ x: 2 }}>
                <button
                  type="button"
                  onClick={() => goCatalog(siteId, b.code)}
                  className={cn(
                    'flex w-full items-center gap-2 rounded-lg px-2.5 py-1.5 text-left text-sm transition-colors',
                    active ? 'bg-surface-3 text-ink' : 'text-ink-dim hover:bg-surface-3/60 hover:text-ink',
                  )}
                >
                  <span className="font-mono text-xs text-accent">/{b.code}/</span>
                  <span className="truncate">{b.title}</span>
                </button>
              </motion.li>
            )
          })}
        </ul>
      </div>

      <form
        className="flex items-center gap-1.5 border-t border-border-soft p-2.5"
        onSubmit={(e) => {
          e.preventDefault()
          const code = manualCode.trim().replace(/^\/|\/$/g, '')
          if (!code) return
          addManualBoard(siteId, code)
          goCatalog(siteId, code)
          setManualCode('')
        }}
      >
        <input
          value={manualCode}
          onChange={(e) => setManualCode(e.target.value)}
          placeholder="Add board by code…"
          className="min-w-0 flex-1 rounded-lg border border-transparent bg-surface-3 px-2.5 py-1.5 text-xs text-ink outline-none transition-colors focus:border-accent/50"
        />
        <button type="submit" className="btn-icon size-7 rounded-lg bg-surface-3 hover:text-accent">
          <Plus size={13} />
        </button>
      </form>
    </div>
  )
}
