import { useEffect, useRef, useState } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { ExternalLink, ImagePlus, KeyRound, Loader2, MessageSquarePlus, RefreshCw, X } from 'lucide-react'
import type { ChanSite } from '../../types'
import { getAdapter } from '../../lib/adapters'
import { fetchCaptcha, postReply, type CaptchaHandle } from '../../lib/lynxchanPosting'
import { openExternalReplyWindow, openAuthWindow } from '../../lib/externalReply'
import { cn } from '../../lib/cn'

export function ReplyComposer({
  site,
  boardCode,
  threadId,
  onPosted,
}: {
  site: ChanSite
  boardCode: string
  threadId: string
  onPosted: () => void
}) {
  const [open, setOpen] = useState(false)
  const [comment, setComment] = useState('')
  const [name, setName] = useState('')
  const [options, setOptions] = useState('')
  const [file, setFile] = useState<File | null>(null)
  const [spoiler, setSpoiler] = useState(false)
  const [captchaAnswer, setCaptchaAnswer] = useState('')
  const [captcha, setCaptcha] = useState<CaptchaHandle | null>(null)
  const [captchaLoading, setCaptchaLoading] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [waitingOnBrowser, setWaitingOnBrowser] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const isLynxchan = site.postEngine === 'lynxchan'

  async function loadCaptcha() {
    setCaptchaLoading(true)
    setError(null)
    try {
      if (captcha) URL.revokeObjectURL(captcha.imageUrl)
      const handle = await fetchCaptcha(site, boardCode)
      setCaptcha(handle)
      setCaptchaAnswer('')
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setCaptchaLoading(false)
    }
  }

  useEffect(() => {
    if (open && isLynxchan && !captcha) loadCaptcha()
    return () => {
      if (captcha) URL.revokeObjectURL(captcha.imageUrl)
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open])

  function reset() {
    setComment('')
    setName('')
    setOptions('')
    setFile(null)
    setSpoiler(false)
    setCaptchaAnswer('')
    setError(null)
    if (captcha) URL.revokeObjectURL(captcha.imageUrl)
    setCaptcha(null)
  }

  async function handleLynxchanSubmit() {
    if (!comment.trim() && !file) {
      setError('Write a comment or attach a file.')
      return
    }
    setSubmitting(true)
    setError(null)
    const result = await postReply(site, boardCode, threadId, {
      comment,
      name: name || undefined,
      options: options || undefined,
      spoiler,
      captchaAnswer,
      file: file ?? undefined,
    })
    setSubmitting(false)
    if (result.ok) {
      reset()
      setOpen(false)
      onPosted()
    } else {
      setError(result.error)
      loadCaptcha()
    }
  }

  async function handleExternalSubmit() {
    const url = getAdapter(site).threadWebUrl(site, boardCode, threadId)
    setWaitingOnBrowser(true)
    await openExternalReplyWindow(url, comment || undefined, () => {
      setWaitingOnBrowser(false)
      onPosted()
    })
  }

  return (
    <div className="border-t border-border-soft">
      <AnimatePresence initial={false}>
        {!open ? (
          <motion.button
            key="closed"
            type="button"
            onClick={() => setOpen(true)}
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="flex w-full items-center justify-center gap-2 px-5 py-4 text-sm font-medium text-ink-dim transition-colors hover:bg-surface-2 hover:text-ink"
          >
            <MessageSquarePlus size={16} />
            Reply to thread
          </motion.button>
        ) : (
          <motion.div
            key="open"
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
            className="overflow-hidden"
          >
            <div className="space-y-3.5 p-6">
              <div className="flex items-center justify-between">
                <p className="text-sm font-semibold text-ink">Reply to thread #{threadId}</p>
                <button
                  type="button"
                  onClick={() => {
                    setOpen(false)
                    reset()
                  }}
                  className="btn-icon size-8"
                >
                  <X size={15} />
                </button>
              </div>

              <div className="flex gap-3">
                <input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Name (optional)"
                  className="input flex-1"
                />
                <input
                  value={options}
                  onChange={(e) => setOptions(e.target.value)}
                  placeholder="Options (e.g. sage)"
                  className="input flex-1"
                />
              </div>

              <textarea
                value={comment}
                onChange={(e) => setComment(e.target.value)}
                placeholder="Comment"
                rows={4}
                className="input resize-none"
              />

              <div className="flex flex-wrap items-center gap-2">
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*,video/*"
                  className="hidden"
                  onChange={(e) => setFile(e.target.files?.[0] ?? null)}
                />
                <button
                  type="button"
                  onClick={() => fileInputRef.current?.click()}
                  className="flex items-center gap-1.5 rounded-lg border border-border-soft bg-surface-2 px-3.5 py-2 text-sm text-ink-dim transition-colors hover:border-border hover:text-ink"
                >
                  <ImagePlus size={13} />
                  {file ? file.name : 'Attach file'}
                </button>
                {file && (
                  <>
                    <button type="button" onClick={() => setFile(null)} className="text-xs text-ink-faint hover:text-red-400">
                      Remove
                    </button>
                    <label className="flex items-center gap-1.5 text-xs text-ink-faint">
                      <input type="checkbox" checked={spoiler} onChange={(e) => setSpoiler(e.target.checked)} className="size-3.5 accent-accent" />
                      Spoiler
                    </label>
                  </>
                )}
              </div>

              {isLynxchan ? (
                <div className="flex flex-wrap items-center gap-3 rounded-xl border border-border-soft bg-surface-2 p-3.5">
                  <div className="flex h-12 min-w-32 items-center justify-center overflow-hidden rounded-lg bg-surface-3">
                    {captchaLoading ? (
                      <Loader2 size={16} className="animate-spin text-ink-faint" />
                    ) : captcha ? (
                      <img src={captcha.imageUrl} alt="Captcha" className="h-full object-contain" />
                    ) : (
                      <span className="px-2 text-[10px] text-ink-faint">No captcha</span>
                    )}
                  </div>
                  <button type="button" onClick={loadCaptcha} title="New captcha" className="btn-icon">
                    <RefreshCw size={13} className={cn(captchaLoading && 'animate-spin')} />
                  </button>
                  <input
                    value={captchaAnswer}
                    onChange={(e) => setCaptchaAnswer(e.target.value)}
                    placeholder="Type the letters"
                    className="input min-w-0 flex-1"
                  />
                </div>
              ) : (
                <div className="flex items-start gap-2.5 rounded-xl border border-border-soft bg-surface-2 p-3.5 text-xs text-ink-faint">
                  <KeyRound size={14} className="mt-0.5 shrink-0" />
                  <p>
                    {site.name} requires solving a verification challenge on their own site. We'll copy your comment to the
                    clipboard and open the thread there — paste it in, verify you're human, and post. Come back here and the
                    thread will refresh.
                    {site.id === '4chan' && (
                      <>
                        {' '}
                        Have a 4chan Pass?{' '}
                        <button type="button" onClick={() => openAuthWindow('https://sys.4chan.org/auth', '4chan Pass sign in')} className="text-accent hover:underline">
                          Sign in
                        </button>{' '}
                        first to skip the captcha.
                      </>
                    )}
                  </p>
                </div>
              )}

              {error && <p className="text-xs text-red-400">{error}</p>}

              <div className="flex justify-end gap-2.5 pt-1.5">
                <button
                  type="button"
                  onClick={() => {
                    setOpen(false)
                    reset()
                  }}
                  className="btn-ghost"
                >
                  Cancel
                </button>
                {isLynxchan ? (
                  <button type="button" disabled={submitting || captchaLoading} onClick={handleLynxchanSubmit} className="btn-primary">
                    {submitting && <Loader2 size={13} className="animate-spin" />}
                    {submitting ? 'Posting…' : 'Post reply'}
                  </button>
                ) : (
                  <button type="button" disabled={waitingOnBrowser} onClick={handleExternalSubmit} className="btn-primary">
                    <ExternalLink size={13} />
                    {waitingOnBrowser ? 'Waiting…' : 'Continue on site'}
                  </button>
                )}
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}
