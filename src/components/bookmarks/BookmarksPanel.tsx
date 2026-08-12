import { motion } from 'motion/react'
import { Bookmark, Trash2 } from 'lucide-react'
import { useBookmarksStore } from '../../store/useBookmarksStore'
import { useSitesStore } from '../../store/useSitesStore'
import { useNavStore } from '../../store/useNavStore'
import { formatRelative } from '../../lib/format'
import { EmptyState } from '../common/EmptyState'

export function BookmarksPanel() {
  const bookmarks = useBookmarksStore((s) => s.bookmarks)
  const remove = useBookmarksStore((s) => s.remove)
  const sites = useSitesStore((s) => s.sites)
  const goThread = useNavStore((s) => s.goThread)

  if (bookmarks.length === 0) {
    return <EmptyState icon={Bookmark} title="No bookmarks yet" description="Tap the bookmark icon on any thread to save it here for quick access." />
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <h1 className="mb-1 text-xl font-semibold text-ink">Bookmarks</h1>
      <p className="mb-6 text-sm text-ink-faint">
        {bookmarks.length} saved thread{bookmarks.length === 1 ? '' : 's'}
      </p>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-4">
        {bookmarks.map((b, i) => {
          const site = sites.find((s) => s.id === b.siteId)
          return (
            <motion.div
              key={`${b.siteId}/${b.boardCode}/${b.threadId}`}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.3, delay: Math.min(i * 0.02, 0.3) }}
              whileHover={{ y: -2 }}
              className="card group relative flex gap-3 overflow-hidden p-3 transition-colors hover:border-border"
            >
              <button
                type="button"
                onClick={() => goThread(b.siteId, b.boardCode, b.threadId)}
                className="flex flex-1 gap-3 text-left"
              >
                <div className="size-16 shrink-0 overflow-hidden rounded-xl bg-surface-3">
                  {b.thumbUrl && <img src={b.thumbUrl} alt="" className="h-full w-full object-cover" />}
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-1.5 text-[10px] font-medium text-ink-faint">
                    <span style={{ color: site?.accent }}>{site?.name ?? b.siteId}</span>
                    <span>/{b.boardCode}/</span>
                  </div>
                  {b.subject && <p className="line-clamp-1 text-[13px] font-semibold text-ink">{b.subject}</p>}
                  <p className="line-clamp-2 text-xs text-ink-dim">{b.excerpt}</p>
                  <p className="mt-1.5 text-[10px] text-ink-faint">Saved {formatRelative(b.addedAt)}</p>
                </div>
              </button>
              <button
                type="button"
                onClick={() => remove(b.siteId, b.boardCode, b.threadId)}
                title="Remove bookmark"
                className="absolute right-2 top-2 flex size-6 items-center justify-center rounded-full bg-black/45 text-white opacity-0 backdrop-blur-sm transition-opacity group-hover:opacity-100"
              >
                <Trash2 size={12} />
              </button>
            </motion.div>
          )
        })}
      </div>
    </div>
  )
}
