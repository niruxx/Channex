import { useState } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { X } from 'lucide-react'
import { useSitesStore } from '../../store/useSitesStore'
import { useNavStore } from '../../store/useNavStore'
import type { ChanSite } from '../../types'

export function AddSiteDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const addCustomSite = useSitesStore((s) => s.addCustomSite)
  const setSite = useNavStore((s) => s.setSite)
  const loadBoards = useSitesStore((s) => s.loadBoards)

  const [name, setName] = useState('')
  const [origin, setOrigin] = useState('https://')
  const [schema, setSchema] = useState<ChanSite['schema']>('yotsuba')
  const [mediaLayout, setMediaLayout] = useState<ChanSite['mediaLayout']>('board-dirs')
  const [postEngine, setPostEngine] = useState<ChanSite['postEngine']>('external')
  const [accent, setAccent] = useState('#6ee7c9')
  const [nsfw, setNsfw] = useState(true)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    setError(null)
    if (!name.trim() || !/^https?:\/\/.+/.test(origin)) {
      setError('Enter a display name and a valid https:// URL.')
      return
    }
    setBusy(true)
    try {
      const site = await addCustomSite({
        id: '',
        name: name.trim(),
        schema,
        mediaLayout,
        origin: origin.trim(),
        accent,
        nsfw,
        postEngine,
      })
      setSite(site.id)
      await loadBoards(site.id, true)
      setName('')
      setOrigin('https://')
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setBusy(false)
    }
  }

  return (
    <AnimatePresence>
      {open && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onMouseDown={(e) => e.target === e.currentTarget && onClose()}
        >
          <motion.form
            onSubmit={handleSubmit}
            initial={{ opacity: 0, scale: 0.95, y: 12 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.96, y: 8 }}
            transition={{ duration: 0.22, ease: [0.16, 1, 0.3, 1] }}
            className="w-[500px] rounded-3xl border border-border bg-surface p-7"
            style={{ boxShadow: 'var(--shadow-lg)' }}
          >
            <div className="mb-6 flex items-center justify-between">
              <h2 className="text-lg font-semibold text-ink">Add imageboard site</h2>
              <button type="button" onClick={onClose} className="btn-icon">
                <X size={16} />
              </button>
            </div>

            <div className="space-y-5">
              <Field label="Display name">
                <input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="e.g. Lainchan"
                  className="input"
                  autoFocus
                />
              </Field>
              <Field label="Site URL">
                <input
                  value={origin}
                  onChange={(e) => setOrigin(e.target.value)}
                  placeholder="https://example.org"
                  className="input"
                />
              </Field>
              <Field label="API schema">
                <select
                  value={schema}
                  onChange={(e) => {
                    const next = e.target.value as ChanSite['schema']
                    setSchema(next)
                    setPostEngine(next === 'lynxchan' ? 'lynxchan' : 'external')
                  }}
                  className="input"
                >
                  <option value="yotsuba">4chan-compatible (vichan / infinity / 4chan-style)</option>
                  <option value="lynxchan">LynxChan native (8chan-family)</option>
                </select>
              </Field>
              {schema === 'yotsuba' && (
                <Field label="Media layout">
                  <select value={mediaLayout} onChange={(e) => setMediaLayout(e.target.value as ChanSite['mediaLayout'])} className="input">
                    <option value="board-dirs">Same-origin /board/src/ + /board/thumb/</option>
                    <option value="flat-cdn">Separate CDN domain, flat files (4chan-style)</option>
                    <option value="file-store">Centralized /file_store/ (8kun-style)</option>
                  </select>
                </Field>
              )}
              <Field label="Replying">
                <select value={postEngine} onChange={(e) => setPostEngine(e.target.value as ChanSite['postEngine'])} className="input">
                  <option value="external">Open the site to reply (handles any captcha)</option>
                  <option value="lynxchan">Post in-app via LynxChan's form API (image captcha)</option>
                </select>
              </Field>
              <div className="flex items-center gap-4">
                <Field label="Accent color" className="flex-1">
                  <input type="color" value={accent} onChange={(e) => setAccent(e.target.value)} className="h-9 w-full cursor-pointer rounded-lg border border-border bg-surface-2" />
                </Field>
                <label className="flex cursor-pointer items-center gap-2 pt-5 text-sm text-ink-dim">
                  <input type="checkbox" checked={nsfw} onChange={(e) => setNsfw(e.target.checked)} className="size-4 rounded accent-accent" />
                  18+ content
                </label>
              </div>
            </div>

            {error && <p className="mt-3 text-sm text-red-400">{error}</p>}

            <div className="mt-6 flex justify-end gap-2">
              <button type="button" onClick={onClose} className="btn-ghost">
                Cancel
              </button>
              <button type="submit" disabled={busy} className="btn-primary">
                {busy ? 'Adding…' : 'Add site'}
              </button>
            </div>
          </motion.form>
        </motion.div>
      )}
    </AnimatePresence>
  )
}

function Field({ label, children, className }: { label: string; children: React.ReactNode; className?: string }) {
  return (
    <label className={`flex flex-col gap-1.5 text-xs font-medium text-ink-dim ${className ?? ''}`}>
      {label}
      {children}
    </label>
  )
}
