import { useState } from 'react'
import { motion } from 'motion/react'
import { Download, Pin, Lock, Play, EyeOff } from 'lucide-react'
import type { ChanSite, Post } from '../../types'
import { CommentText } from './CommentText'
import { formatBytes, formatDate } from '../../lib/format'
import { useLightboxStore } from '../../store/useLightboxStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { useDownloadsStore } from '../../store/useDownloadsStore'
import { pickDownloadDir } from '../../lib/download'
import { cn } from '../../lib/cn'
import { BirthdayHat } from '../common/BirthdayHat'
import { useShowBirthdayHats } from '../../hooks/useShowBirthdayHats'

export function PostCard({
  post,
  site,
  highlighted,
  backlinks,
  onQuoteClick,
}: {
  post: Post
  site: ChanSite
  highlighted?: boolean
  backlinks?: string[]
  onQuoteClick?: (id: string) => void
}) {
  const openLightbox = useLightboxStore((s) => s.open)
  const blurNsfw = useSettingsStore((s) => s.blurNsfw)
  const startJob = useDownloadsStore((s) => s.startJob)
  const downloadDir = useSettingsStore((s) => s.downloadDir)
  const setDownloadDir = useSettingsStore((s) => s.setDownloadDir)
  const [revealed, setRevealed] = useState<Record<number, boolean>>({})
  const showHat = useShowBirthdayHats()

  return (
    <motion.div
      id={`post-${post.id}`}
      layout="position"
      initial={{ opacity: 0, y: 6 }}
      animate={{
        opacity: 1,
        y: 0,
        backgroundColor: highlighted ? 'color-mix(in oklab, var(--color-accent) 14%, var(--color-surface-2))' : 'var(--color-surface-2)',
      }}
      transition={{ duration: 0.3 }}
      className={cn('relative scroll-mt-16 rounded-2xl border p-5', highlighted ? 'border-accent/50' : 'border-border-soft')}
      style={{ boxShadow: 'var(--shadow-sm)' }}
    >
      {showHat && <BirthdayHat size={26} rotate={-20} className="absolute -top-3 left-3 z-10" />}

      <div className="mb-2.5 flex flex-wrap items-center gap-x-3 gap-y-1.5 text-[13px]">
        {post.subject && <span className="font-semibold text-ink">{post.subject}</span>}
        <span className="font-semibold text-emerald-300/90">{post.name}</span>
        {post.tripcode && <span className="text-violet-300/80">{post.tripcode}</span>}
        {post.capcode && (
          <span className="rounded-full bg-accent/20 px-1.5 py-0.5 text-[11px] font-semibold uppercase tracking-wide text-accent">{post.capcode}</span>
        )}
        {post.countryCode && <span className="text-ink-faint" title={post.countryName}>{post.countryCode}</span>}
        <span className="text-ink-faint">{formatDate(post.timestamp)}</span>
        <span className="text-ink-faint">No.{post.id}</span>
        {post.sticky && <Pin size={12} className="text-accent" />}
        {post.closed && <Lock size={12} className="text-red-400" />}
      </div>

      {backlinks && backlinks.length > 0 && (
        <div className="mb-2.5 flex flex-wrap gap-1.5 text-xs">
          {backlinks.map((id) => (
            <button key={id} type="button" onClick={() => onQuoteClick?.(id)} className="rounded-md bg-surface-3 px-1.5 py-0.5 text-sky-400 transition-colors hover:bg-surface-4 hover:underline">
              &gt;&gt;{id}
            </button>
          ))}
        </div>
      )}

      {post.files.length > 0 && (
        <div className="mb-3.5 flex flex-wrap gap-3">
          {post.files.map((file, i) => {
            const shouldBlur = (site.nsfw || file.spoiler) && blurNsfw && !revealed[i]
            return (
              <div key={i} className="group relative">
                <button
                  type="button"
                  onClick={() => {
                    if (shouldBlur) {
                      setRevealed((r) => ({ ...r, [i]: true }))
                      return
                    }
                    openLightbox(post.files, i)
                  }}
                  className="block overflow-hidden rounded-xl border border-border-soft bg-surface-3"
                  style={{ boxShadow: 'var(--shadow-sm)' }}
                >
                  <img
                    src={file.thumbUrl}
                    alt={file.name}
                    className={cn('h-48 w-auto max-w-64 object-cover transition-transform duration-200 group-hover:scale-[1.03]', shouldBlur && 'blur-xl')}
                  />
                  {file.isVideo && !shouldBlur && (
                    <span className="absolute inset-0 flex items-center justify-center bg-black/20">
                      <Play size={24} className="text-white drop-shadow" fill="white" />
                    </span>
                  )}
                  {shouldBlur && (
                    <span className="absolute inset-0 flex flex-col items-center justify-center gap-1 text-xs font-medium text-white">
                      <EyeOff size={15} />
                      Reveal
                    </span>
                  )}
                </button>
                {!shouldBlur && (
                  <button
                    type="button"
                    title="Download file"
                    onClick={async () => {
                      let dir = downloadDir
                      if (!dir) {
                        dir = await pickDownloadDir()
                        if (!dir) return
                        await setDownloadDir(dir)
                      }
                      await startJob(file.name, dir, [{ url: file.url, fileName: file.name }])
                    }}
                    className="absolute right-2 top-2 flex size-7 items-center justify-center rounded-full bg-black/55 text-white opacity-0 backdrop-blur-sm transition-opacity group-hover:opacity-100"
                  >
                    <Download size={13} />
                  </button>
                )}
                <div className="mt-2 max-w-64 truncate text-[11px] text-ink-faint">
                  {file.name} {file.size ? `· ${formatBytes(file.size)}` : ''}
                </div>
              </div>
            )
          })}
        </div>
      )}

      <CommentText html={post.commentHtml} onQuoteClick={onQuoteClick} />
    </motion.div>
  )
}
