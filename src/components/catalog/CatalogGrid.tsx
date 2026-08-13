import { useEffect, useMemo, useState } from 'react'
import { AlertTriangle, RefreshCw, SearchX } from 'lucide-react'
import { motion } from 'motion/react'
import { useCatalog } from '../../hooks/useCatalog'
import { useSitesStore } from '../../store/useSitesStore'
import { useNavStore } from '../../store/useNavStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { ThreadCard } from './ThreadCard'
import { ThreadListRow } from './ThreadListRow'
import { ViewModeToggle } from './ViewModeToggle'
import { CatalogToolbar } from './CatalogToolbar'
import { filterThreads, sortThreads, type CatalogSort } from './sort'
import { CatalogGridSkeleton } from '../common/Skeletons'
import { EmptyState } from '../common/EmptyState'
import { htmlToText } from '../../lib/sanitize'
import { cn } from '../../lib/cn'

export function CatalogGrid({ siteId, boardCode }: { siteId: string; boardCode: string }) {
  const site = useSitesStore((s) => s.getSite(siteId))
  const boardTitle = useSitesStore((s) => s.boards[siteId]?.find((b) => b.code === boardCode)?.title)
  const goThread = useNavStore((s) => s.goThread)
  const viewMode = useSettingsStore((s) => s.catalogViewMode)
  const { threads, loading, error, reload } = useCatalog(siteId, boardCode)
  const [query, setQuery] = useState('')
  const [sort, setSort] = useState<CatalogSort>('bump')

  // A search typed on one board shouldn't silently hide threads on the next.
  useEffect(() => {
    setQuery('')
  }, [siteId, boardCode])

  useEffect(() => {
    function onRefresh() {
      reload()
    }
    window.addEventListener('imageboarder:refresh', onRefresh)
    return () => window.removeEventListener('imageboarder:refresh', onRefresh)
  }, [reload])

  const visible = useMemo(() => {
    if (!threads) return null
    return sortThreads(filterThreads(threads, query, (p) => htmlToText(p.commentHtml)), sort)
  }, [threads, query, sort])

  if (!site) return null

  if (error && !threads) {
    return (
      <EmptyState
        icon={AlertTriangle}
        title="Couldn't load this board"
        description={error}
        action={
          <button type="button" onClick={reload} className="btn-secondary mt-2">
            <RefreshCw size={13} /> Retry
          </button>
        }
      />
    )
  }

  if (!threads && loading) {
    return <CatalogGridSkeleton />
  }

  if (threads && threads.length === 0) {
    return <EmptyState icon={AlertTriangle} title="No threads found" description="This board appears to be empty right now." />
  }

  const filtering = query.trim().length > 0

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 border-b border-border-soft px-6 py-3.5">
        <div className="flex min-w-0 items-baseline gap-2">
          <h1 className="shrink-0 text-base font-semibold text-ink">/{boardCode}/</h1>
          {boardTitle && <span className="truncate text-sm text-ink-dim">{boardTitle}</span>}
        </div>
        <span className="shrink-0 rounded-full bg-surface-3 px-2 py-0.5 text-[11px] font-medium text-ink-faint">
          {filtering ? `${visible?.length ?? 0} of ${threads?.length ?? 0}` : `${threads?.length ?? 0} threads`}
        </span>
        <div className="ml-auto flex shrink-0 items-center gap-2">
          <CatalogToolbar query={query} onQueryChange={setQuery} sort={sort} onSortChange={setSort} />
          <ViewModeToggle />
          <button type="button" onClick={reload} title="Refresh catalog (R)" className="btn-icon">
            <RefreshCw size={15} className={cn(loading && 'animate-spin')} />
          </button>
        </div>
      </div>

      {visible && visible.length === 0 ? (
        <EmptyState
          icon={SearchX}
          title="No matching threads"
          description={`Nothing on /${boardCode}/ matches "${query.trim()}".`}
          action={
            <button type="button" onClick={() => setQuery('')} className="btn-secondary mt-2">
              Clear search
            </button>
          }
        />
      ) : viewMode === 'list' ? (
        <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-6">
          {visible?.map((t, i) => (
            <ThreadListRow key={t.id} post={t} site={site} boardCode={boardCode} index={i} onOpen={() => goThread(siteId, boardCode, t.threadId)} />
          ))}
        </div>
      ) : (
        <motion.div
          layout
          className={cn(
            'grid min-h-0 flex-1 auto-rows-min overflow-y-auto',
            viewMode === 'compact' ? 'grid-cols-[repeat(auto-fill,minmax(130px,1fr))] gap-3 p-5' : 'grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-5 p-6',
          )}
        >
          {visible?.map((t, i) => (
            <ThreadCard
              key={t.id}
              post={t}
              site={site}
              boardCode={boardCode}
              index={i}
              compact={viewMode === 'compact'}
              onOpen={() => goThread(siteId, boardCode, t.threadId)}
            />
          ))}
        </motion.div>
      )}
    </div>
  )
}
