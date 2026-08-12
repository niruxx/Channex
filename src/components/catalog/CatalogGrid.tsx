import { AlertTriangle, RefreshCw } from 'lucide-react'
import { motion } from 'motion/react'
import { useCatalog } from '../../hooks/useCatalog'
import { useSitesStore } from '../../store/useSitesStore'
import { useNavStore } from '../../store/useNavStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { ThreadCard } from './ThreadCard'
import { ThreadListRow } from './ThreadListRow'
import { ViewModeToggle } from './ViewModeToggle'
import { CatalogGridSkeleton } from '../common/Skeletons'
import { EmptyState } from '../common/EmptyState'
import { cn } from '../../lib/cn'

export function CatalogGrid({ siteId, boardCode }: { siteId: string; boardCode: string }) {
  const site = useSitesStore((s) => s.getSite(siteId))
  const boardTitle = useSitesStore((s) => s.boards[siteId]?.find((b) => b.code === boardCode)?.title)
  const goThread = useNavStore((s) => s.goThread)
  const viewMode = useSettingsStore((s) => s.catalogViewMode)
  const { threads, loading, error, reload } = useCatalog(siteId, boardCode)

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

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 border-b border-border-soft px-6 py-4">
        <div className="flex items-baseline gap-2">
          <h1 className="text-base font-semibold text-ink">/{boardCode}/</h1>
          {boardTitle && <span className="text-sm text-ink-dim">{boardTitle}</span>}
        </div>
        <span className="rounded-full bg-surface-3 px-2 py-0.5 text-[11px] font-medium text-ink-faint">{threads?.length ?? 0} threads</span>
        <div className="ml-auto flex items-center gap-2">
          <ViewModeToggle />
          <button type="button" onClick={reload} className="btn-ghost px-2.5 py-1.5 text-xs">
            <RefreshCw size={12} className={cn(loading && 'animate-spin')} />
            Refresh
          </button>
        </div>
      </div>

      {viewMode === 'list' ? (
        <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto p-6">
          {threads?.map((t, i) => (
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
          {threads?.map((t, i) => (
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
