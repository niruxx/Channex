import { useEffect, useMemo, useRef, useState } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { AlertTriangle, ArrowUp, Bookmark, DownloadCloud, ExternalLink, RefreshCw } from 'lucide-react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { join } from '@tauri-apps/api/path'
import { useThread } from '../../hooks/useThread'
import { useSitesStore } from '../../store/useSitesStore'
import { useBookmarksStore } from '../../store/useBookmarksStore'
import { useDownloadsStore } from '../../store/useDownloadsStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { getAdapter } from '../../lib/adapters'
import { pickDownloadDir } from '../../lib/download'
import { sanitizeFileName } from '../../lib/format'
import { htmlToText } from '../../lib/sanitize'
import { PostCard } from './PostCard'
import { ReplyComposer } from './ReplyComposer'
import { PostSkeleton } from '../common/Skeletons'
import { EmptyState } from '../common/EmptyState'
import { cn } from '../../lib/cn'

export function ThreadView({ siteId, boardCode, threadId }: { siteId: string; boardCode: string; threadId: string }) {
  const site = useSitesStore((s) => s.getSite(siteId))
  const { data, loading, error, reload } = useThread(siteId, boardCode, threadId)
  const isBookmarked = useBookmarksStore((s) => s.isBookmarked(siteId, boardCode, threadId))
  const toggleBookmark = useBookmarksStore((s) => s.toggle)
  const markSeen = useBookmarksStore((s) => s.markSeen)
  const startJob = useDownloadsStore((s) => s.startJob)
  const downloadDir = useSettingsStore((s) => s.downloadDir)
  const setDownloadDir = useSettingsStore((s) => s.setDownloadDir)
  const [highlighted, setHighlighted] = useState<string | null>(null)
  const [downloadingAll, setDownloadingAll] = useState(false)
  const [scrolled, setScrolled] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)

  const replyCount = data?.replies.length

  useEffect(() => {
    if (!highlighted) return
    const t = setTimeout(() => setHighlighted(null), 1400)
    return () => clearTimeout(t)
  }, [highlighted])

  // Reading the thread is what clears its "new replies" badge in the catalog.
  useEffect(() => {
    if (replyCount == null) return
    markSeen(siteId, boardCode, threadId, replyCount)
  }, [markSeen, siteId, boardCode, threadId, replyCount])

  useEffect(() => {
    function onRefresh() {
      reload()
    }
    window.addEventListener('imageboarder:refresh', onRefresh)
    return () => window.removeEventListener('imageboarder:refresh', onRefresh)
  }, [reload])

  const backlinkMap = useMemo(() => {
    const map = new Map<string, string[]>()
    if (!data) return map
    const all = [data.op, ...data.replies]
    for (const post of all) {
      const ids = new Set<string>()
      const re = /data-quote-id="(\d+)"/g
      let m: RegExpExecArray | null
      while ((m = re.exec(post.commentHtml))) ids.add(m[1])
      for (const id of ids) {
        const arr = map.get(id) ?? []
        arr.push(post.id)
        map.set(id, arr)
      }
    }
    return map
  }, [data])

  function handleQuoteClick(id: string) {
    const el = document.getElementById(`post-${id}`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' })
      setHighlighted(id)
    }
  }

  async function handleDownloadAll() {
    if (!data || !site) return
    let dir = downloadDir
    if (!dir) {
      dir = await pickDownloadDir()
      if (!dir) return
      await setDownloadDir(dir)
    }
    const all = [data.op, ...data.replies]
    const files = all.flatMap((p) => p.files.map((f) => ({ url: f.url, fileName: sanitizeFileName(f.name) })))
    if (files.length === 0) return
    setDownloadingAll(true)
    try {
      const subDir = await join(dir, `${site.id}-${boardCode}-${threadId}`)
      await startJob(`Thread #${threadId}`, subDir, files)
    } finally {
      setDownloadingAll(false)
    }
  }

  if (!site) return null

  if (error && !data) {
    return (
      <EmptyState
        icon={AlertTriangle}
        title="Couldn't load this thread"
        description={error}
        action={
          <button type="button" onClick={reload} className="btn-secondary mt-2">
            <RefreshCw size={13} /> Retry
          </button>
        }
      />
    )
  }

  if (!data && loading) {
    return (
      <div className="flex flex-col gap-4 overflow-y-auto px-6 py-5">
        {Array.from({ length: 6 }).map((_, i) => (
          <PostSkeleton key={i} />
        ))}
      </div>
    )
  }

  if (!data) return null

  const totalFiles = [data.op, ...data.replies].reduce((n, p) => n + p.files.length, 0)

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div className="flex items-center gap-3 border-b border-border-soft px-6 py-4">
        <h1 className="line-clamp-1 text-base font-semibold text-ink">{data.op.subject || `Thread #${threadId}`}</h1>
        <span className="shrink-0 rounded-full bg-surface-3 px-2 py-0.5 text-[11px] font-medium text-ink-faint">{data.replies.length + 1} posts</span>
        <div className="ml-auto flex shrink-0 items-center gap-1">
          <button
            type="button"
            onClick={() =>
              toggleBookmark({
                siteId,
                boardCode,
                threadId,
                subject: data.op.subject,
                excerpt: htmlToText(data.op.commentHtml).slice(0, 140),
                thumbUrl: data.op.files[0]?.thumbUrl,
                addedAt: Date.now(),
                lastSeenReplyCount: data.replies.length,
              })
            }
            title="Bookmark thread"
            className={cn('btn-icon', isBookmarked && 'text-accent hover:text-accent')}
          >
            <Bookmark size={15} fill={isBookmarked ? 'currentColor' : 'none'} />
          </button>
          {totalFiles > 0 && (
            <button
              type="button"
              disabled={downloadingAll}
              onClick={handleDownloadAll}
              title="Download all media in thread"
              className="btn-ghost px-2.5 py-1.5 text-xs disabled:opacity-50"
            >
              <DownloadCloud size={14} />
              {downloadingAll ? 'Starting…' : `Download all (${totalFiles})`}
            </button>
          )}
          <button
            type="button"
            onClick={() => openUrl(getAdapter(site).threadWebUrl(site, boardCode, threadId)).catch(() => {})}
            title="Open in browser"
            className="btn-icon"
          >
            <ExternalLink size={15} />
          </button>
          <button type="button" onClick={reload} title="Refresh thread (R)" className="btn-icon">
            <RefreshCw size={15} className={cn(loading && 'animate-spin')} />
          </button>
        </div>
      </div>

      <div
        ref={scrollRef}
        onScroll={(e) => setScrolled(e.currentTarget.scrollTop > 600)}
        className="relative flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-6 py-5"
      >
        <PostCard post={data.op} site={site} highlighted={highlighted === data.op.id} backlinks={backlinkMap.get(data.op.id)} onQuoteClick={handleQuoteClick} />
        {data.replies.map((r) => (
          <PostCard key={r.id} post={r} site={site} highlighted={highlighted === r.id} backlinks={backlinkMap.get(r.id)} onQuoteClick={handleQuoteClick} />
        ))}
      </div>

      <AnimatePresence>
        {scrolled && (
          <motion.button
            type="button"
            initial={{ opacity: 0, scale: 0.8, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.8, y: 8 }}
            transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
            onClick={() => scrollRef.current?.scrollTo({ top: 0, behavior: 'smooth' })}
            title="Back to top"
            className="absolute bottom-24 right-6 z-20 flex size-10 items-center justify-center rounded-full bg-surface-4 text-ink shadow-lg transition-colors hover:text-accent"
          >
            <ArrowUp size={17} />
          </motion.button>
        )}
      </AnimatePresence>

      {!data.op.closed && <ReplyComposer site={site} boardCode={boardCode} threadId={threadId} onPosted={reload} />}
    </div>
  )
}
