import { useState } from 'react'
import { motion } from 'motion/react'
import { Bookmark, MessageSquare, Image as ImageIcon, Pin, Lock, EyeOff, MessageSquareOff } from 'lucide-react'
import type { ChanSite, Post } from '../../types'
import { htmlToText } from '../../lib/sanitize'
import { formatRelative } from '../../lib/format'
import { useBookmarksStore } from '../../store/useBookmarksStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { cn } from '../../lib/cn'
import { BirthdayHat } from '../common/BirthdayHat'
import { useShowBirthdayHats } from '../../hooks/useShowBirthdayHats'

export function ThreadListRow({
  post,
  site,
  boardCode,
  onOpen,
  index,
}: {
  post: Post
  site: ChanSite
  boardCode: string
  onOpen: () => void
  index: number
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
      initial={{ opacity: 0, y: 8 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.25, delay: Math.min(index * 0.015, 0.3), ease: [0.16, 1, 0.3, 1] }}
      className="card group flex items-center gap-4 p-3 text-left transition-colors duration-200 hover:border-border"
    >
      <div className="relative size-20 shrink-0 overflow-hidden rounded-xl bg-surface-3">
        {thumb ? (
          <img
            src={thumb.thumbUrl}
            alt=""
            loading="lazy"
            className={cn('h-full w-full object-cover', shouldBlur && 'scale-110 blur-md')}
            onError={(e) => {
              ;(e.currentTarget as HTMLImageElement).style.visibility = 'hidden'
            }}
          />
        ) : (
          <div className="flex h-full items-center justify-center text-ink-faint">
            <MessageSquareOff size={18} strokeWidth={1.5} />
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
            className="absolute inset-0 flex items-center justify-center bg-black/35 text-white"
          >
            <EyeOff size={14} />
          </div>
        )}
      </div>

      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          {showHat && <BirthdayHat size={17} rotate={-14} className="shrink-0" />}
          {post.sticky && <Pin size={12} className="shrink-0 text-accent" />}
          {post.closed && <Lock size={12} className="shrink-0 text-red-400" />}
          {post.subject && <span className="line-clamp-1 text-sm font-semibold text-ink">{post.subject}</span>}
        </div>
        <p className="line-clamp-1 text-sm text-ink-dim">{excerpt || <span className="italic text-ink-faint">No comment</span>}</p>
      </div>

      <div className="flex shrink-0 items-center gap-5 text-xs font-medium text-ink-faint">
        {newReplies > 0 && (
          <span className="rounded-full bg-accent px-1.5 py-0.5 text-[11px] font-bold text-accent-ink" title={`${newReplies} new since you last read this`}>
            +{newReplies}
          </span>
        )}
        <span className="flex items-center gap-1.5">
          <MessageSquare size={13} /> {post.replyCount ?? 0}
        </span>
        <span className="flex items-center gap-1.5">
          <ImageIcon size={13} /> {post.imageCount ?? 0}
        </span>
        <span className="hidden w-16 text-right sm:inline">{formatRelative(post.timestamp)}</span>
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
          className={cn('btn-icon size-8', isBookmarked && 'text-accent hover:text-accent')}
        >
          <Bookmark size={14} fill={isBookmarked ? 'currentColor' : 'none'} />
        </button>
      </div>
    </motion.button>
  )
}
