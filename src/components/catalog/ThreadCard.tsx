import { useState } from 'react'
import { motion } from 'motion/react'
import { Bookmark, MessageSquare, Image as ImageIcon, Pin, Lock, EyeOff } from 'lucide-react'
import type { ChanSite, Post } from '../../types'
import { htmlToText } from '../../lib/sanitize'
import { formatRelative } from '../../lib/format'
import { useBookmarksStore } from '../../store/useBookmarksStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { cn } from '../../lib/cn'
import { BirthdayHat } from '../common/BirthdayHat'
import { useShowBirthdayHats } from '../../hooks/useShowBirthdayHats'

export function ThreadCard({
  post,
  site,
  boardCode,
  onOpen,
  index,
  compact = false,
}: {
  post: Post
  site: ChanSite
  boardCode: string
  onOpen: () => void
  index: number
  compact?: boolean
}) {
  const isBookmarked = useBookmarksStore((s) => s.isBookmarked(site.id, boardCode, post.threadId))
  const lastSeen = useBookmarksStore((s) => s.getBookmark(site.id, boardCode, post.threadId)?.lastSeenReplyCount)
  const toggle = useBookmarksStore((s) => s.toggle)
  const blurNsfw = useSettingsStore((s) => s.blurNsfw)
  const [revealed, setRevealed] = useState(false)
  const showHat = useShowBirthdayHats()

  const newReplies = lastSeen == null ? 0 : Math.max(0, (post.replyCount ?? 0) - lastSeen)

  const thumb = post.files[0]
  const excerpt = htmlToText(post.commentHtml)
  const shouldBlur = (site.nsfw || thumb?.spoiler) && blurNsfw && !revealed

  return (
    <motion.button
      type="button"
      onClick={onOpen}
      layout
      initial={{ opacity: 0, y: 14, scale: 0.97 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      transition={{ duration: 0.35, delay: Math.min(index * 0.025, 0.4), ease: [0.16, 1, 0.3, 1] }}
      whileHover={{ y: -4, boxShadow: 'var(--shadow-lg)' }}
      className="card group relative flex flex-col overflow-hidden text-left transition-colors duration-200 hover:border-border"
    >
      <div className="relative aspect-square w-full overflow-hidden bg-surface-3">
        {thumb ? (
          <motion.img
            src={thumb.thumbUrl}
            alt=""
            loading="lazy"
            className={cn('h-full w-full object-cover transition-all duration-300 group-hover:scale-[1.04]', shouldBlur && 'scale-110 blur-xl')}
            onError={(e) => {
              ;(e.currentTarget as HTMLImageElement).style.visibility = 'hidden'
            }}
          />
        ) : (
          <div className="flex h-full items-center justify-center text-ink-faint">
            <MessageSquare size={22} strokeWidth={1.5} />
          </div>
        )}
        {shouldBlur && (
          <div
            role="button"
            tabIndex={-1}
            onClick={(e) => {
              e.stopPropagation()
              setRevealed(true)
            }}
            className="absolute inset-0 flex flex-col items-center justify-center gap-1.5 bg-black/35 text-xs font-medium text-white"
          >
            <EyeOff size={17} />
            Tap to reveal
          </div>
        )}
        <div className="absolute inset-x-0 top-0 flex items-start justify-between p-2">
          <div className="flex items-start gap-1">
            {showHat && <BirthdayHat size={20} rotate={-14} className="-ml-0.5 -mt-0.5" />}
            {post.sticky && (
              <span className="flex size-6 items-center justify-center rounded-full bg-black/55 text-accent backdrop-blur-sm">
                <Pin size={11} />
              </span>
            )}
            {post.closed && (
              <span className="flex size-6 items-center justify-center rounded-full bg-black/55 text-red-400 backdrop-blur-sm">
                <Lock size={11} />
              </span>
            )}
          </div>
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation()
              toggle({
                siteId: site.id,
                boardCode,
                threadId: post.threadId,
                subject: post.subject,
                excerpt: excerpt.slice(0, 140),
                thumbUrl: thumb?.thumbUrl,
                addedAt: Date.now(),
                lastSeenReplyCount: post.replyCount,
              })
            }}
            className={cn(
              'flex size-6 items-center justify-center rounded-full bg-black/55 text-white opacity-0 backdrop-blur-sm transition-opacity group-hover:opacity-100',
              isBookmarked && 'opacity-100 text-accent',
            )}
          >
            <Bookmark size={12} fill={isBookmarked ? 'currentColor' : 'none'} />
          </button>
        </div>
      </div>

      <div className={cn('flex flex-1 flex-col gap-2', compact ? 'p-2.5' : 'p-4')}>
        {post.subject && <p className="line-clamp-1 text-sm font-semibold text-ink">{post.subject}</p>}
        {!compact && (
          <p className="line-clamp-3 min-h-[3.2em] text-[13px] leading-relaxed text-ink-dim">
            {excerpt || <span className="italic text-ink-faint">No comment</span>}
          </p>
        )}
        <div className={cn('mt-auto flex items-center gap-3 text-[11px] font-medium text-ink-faint', !compact && 'pt-2')}>
          <span className="flex items-center gap-1">
            <MessageSquare size={12} /> {post.replyCount ?? 0}
          </span>
          <span className="flex items-center gap-1">
            <ImageIcon size={12} /> {post.imageCount ?? 0}
          </span>
          {newReplies > 0 && (
            <span className="rounded-full bg-accent px-1.5 py-0.5 text-[10px] font-bold text-accent-ink" title={`${newReplies} new since you last read this`}>
              +{newReplies}
            </span>
          )}
          {!compact && <span className="ml-auto">{formatRelative(post.timestamp)}</span>}
        </div>
      </div>
    </motion.button>
  )
}
