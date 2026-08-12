import { useEffect, useState } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { AlertTriangle, ChevronLeft, ChevronRight, Download, ExternalLink, Loader2, X, ZoomIn, ZoomOut, FolderOpen } from 'lucide-react'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useLightboxStore } from '../../store/useLightboxStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { useDownloadsStore } from '../../store/useDownloadsStore'
import { pickDownloadDir, uniqueDestPath, downloadToFile, revealInFolder } from '../../lib/download'
import { getBytes } from '../../lib/http'
import { formatBytes, mimeTypeForExt } from '../../lib/format'
import { cn } from '../../lib/cn'

type LoadStatus = 'loading' | 'loaded' | 'error'

export function MediaLightbox() {
  const { isOpen, files, index, close, next, prev } = useLightboxStore()
  const downloadDir = useSettingsStore((s) => s.downloadDir)
  const setDownloadDir = useSettingsStore((s) => s.setDownloadDir)
  const muteWebmsByDefault = useSettingsStore((s) => s.muteWebmsByDefault)
  const startJob = useDownloadsStore((s) => s.startJob)
  const [zoomed, setZoomed] = useState(false)
  const [savedPath, setSavedPath] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)
  const [status, setStatus] = useState<LoadStatus>('loading')
  const [mediaSrc, setMediaSrc] = useState<string | null>(null)
  const [retryToken, setRetryToken] = useState(0)

  const file = files[index]

  useEffect(() => {
    setZoomed(false)
    setSavedPath(null)
  }, [index, isOpen])

  // Full-resolution media is fetched through the same Rust-side HTTP client used for
  // downloads (rather than a direct webview <img>/<video> network request) and shown via
  // a blob: URL — the webview's own request path has proven unreliable for some hosts here.
  useEffect(() => {
    if (!isOpen || !file) return
    let cancelled = false
    let objectUrl: string | null = null
    setStatus('loading')
    setMediaSrc(null)

    getBytes(file.url, { timeoutMs: 120_000 })
      .then((bytes) => {
        if (cancelled) return
        const blob = new Blob([bytes.buffer as ArrayBuffer], { type: mimeTypeForExt(file.ext) })
        objectUrl = URL.createObjectURL(blob)
        setMediaSrc(objectUrl)
        setStatus('loaded')
      })
      .catch(() => {
        if (!cancelled) setStatus('error')
      })

    return () => {
      cancelled = true
      if (objectUrl) URL.revokeObjectURL(objectUrl)
    }
  }, [file?.url, isOpen, retryToken])

  useEffect(() => {
    if (!isOpen) return
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') close()
      else if (e.key === 'ArrowRight') next()
      else if (e.key === 'ArrowLeft') prev()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [isOpen, close, next, prev])

  if (!file) return null

  function handleRetry() {
    setRetryToken((t) => t + 1)
  }

  async function handleSave() {
    let dir = downloadDir
    if (!dir) {
      dir = await pickDownloadDir()
      if (!dir) return
      await setDownloadDir(dir)
    }
    setSaving(true)
    try {
      const dest = await uniqueDestPath(dir, file.name)
      await downloadToFile(file.url, dest)
      setSavedPath(dest)
    } finally {
      setSaving(false)
    }
  }

  async function handleSaveToQueue() {
    let dir = downloadDir
    if (!dir) {
      dir = await pickDownloadDir()
      if (!dir) return
      await setDownloadDir(dir)
    }
    await startJob(file.name, dir, [{ url: file.url, fileName: file.name }])
  }

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          className="fixed inset-0 z-[60] flex flex-col bg-black/85 backdrop-blur-md"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onMouseDown={(e) => e.target === e.currentTarget && close()}
        >
          <div className="flex items-center justify-between px-6 py-4 text-sm text-white/80">
            <div className="flex items-center gap-3">
              <span className="truncate max-w-md">{file.name}</span>
              <span className="text-white/40">
                {file.width && file.height ? `${file.width}×${file.height}` : ''} {file.size ? `· ${formatBytes(file.size)}` : ''}
              </span>
            </div>
            <div className="flex items-center gap-1">
              {!file.isVideo && status === 'loaded' && (
                <IconBtn onClick={() => setZoomed((z) => !z)} title={zoomed ? 'Zoom out' : 'Zoom in'}>
                  {zoomed ? <ZoomOut size={16} /> : <ZoomIn size={16} />}
                </IconBtn>
              )}
              <IconBtn onClick={handleSaveToQueue} title="Add to download queue">
                <Download size={16} />
              </IconBtn>
              {savedPath && (
                <IconBtn onClick={() => revealInFolder(savedPath)} title="Reveal saved file">
                  <FolderOpen size={16} />
                </IconBtn>
              )}
              <IconBtn onClick={close} title="Close (Esc)">
                <X size={16} />
              </IconBtn>
            </div>
          </div>

          <div className="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden px-4 pb-4">
            {files.length > 1 && (
              <>
                <NavArrow side="left" onClick={prev} />
                <NavArrow side="right" onClick={next} />
              </>
            )}

            {status === 'loading' && (
              <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
                <Loader2 size={28} className="animate-spin text-white/60" />
              </div>
            )}

            {status === 'error' && (
              <div className="flex flex-col items-center gap-3 text-center text-white/80">
                <AlertTriangle size={28} className="text-amber-400" />
                <p className="max-w-sm text-sm">
                  Couldn't load this file. The source site may be slow, unreachable, or blocking direct access.
                </p>
                <div className="flex gap-2">
                  <button type="button" onClick={handleRetry} className="btn-secondary">
                    Retry
                  </button>
                  <button type="button" onClick={() => openUrl(file.url).catch(() => {})} className="btn-secondary">
                    <ExternalLink size={13} /> Open in browser
                  </button>
                </div>
              </div>
            )}

            {status === 'loaded' && mediaSrc && (
              <AnimatePresence mode="wait" initial={false}>
                <motion.div
                  key={file.url}
                  drag={!zoomed && files.length > 1 ? 'x' : false}
                  dragConstraints={{ left: 0, right: 0 }}
                  dragElastic={0.6}
                  onDragEnd={(_, info) => {
                    if (info.offset.x < -80) next()
                    else if (info.offset.x > 80) prev()
                  }}
                  initial={{ opacity: 0, scale: 0.96 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.97 }}
                  transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
                  className="flex max-h-full max-w-full items-center justify-center"
                >
                  {file.isVideo ? (
                    <video
                      src={mediaSrc}
                      controls
                      autoPlay
                      muted={muteWebmsByDefault}
                      loop
                      playsInline
                      onError={() => setStatus('error')}
                      className="max-h-[80vh] max-w-[90vw] rounded-lg shadow-2xl"
                    />
                  ) : (
                    <img
                      src={mediaSrc}
                      alt={file.name}
                      onClick={() => setZoomed((z) => !z)}
                      onError={() => setStatus('error')}
                      className={cn(
                        'max-h-[80vh] max-w-[90vw] select-none rounded-lg object-contain shadow-2xl transition-transform duration-300',
                        zoomed ? 'max-h-none max-w-none scale-[1.9] cursor-zoom-out' : 'cursor-zoom-in',
                      )}
                      draggable={false}
                    />
                  )}
                </motion.div>
              </AnimatePresence>
            )}
          </div>

          {files.length > 1 && (
            <div className="flex items-center justify-center gap-1 pb-4">
              {files.map((_, i) => (
                <span key={i} className={cn('h-1.5 rounded-full transition-all', i === index ? 'w-4 bg-white' : 'w-1.5 bg-white/30')} />
              ))}
            </div>
          )}

          <button
            type="button"
            onClick={handleSave}
            disabled={saving}
            className="absolute bottom-6 right-6 flex items-center gap-2 rounded-full bg-accent px-4 py-2.5 text-xs font-semibold text-accent-ink shadow-lg transition-transform active:scale-95 disabled:opacity-60"
          >
            <Download size={13} />
            {saving ? 'Saving…' : 'Save to disk'}
          </button>
        </motion.div>
      )}
    </AnimatePresence>
  )
}

function IconBtn({ children, onClick, title }: { children: React.ReactNode; onClick: () => void; title: string }) {
  return (
    <button type="button" onClick={onClick} title={title} className="flex size-8 items-center justify-center rounded-full text-white/80 transition-colors hover:bg-white/10 hover:text-white">
      {children}
    </button>
  )
}

function NavArrow({ side, onClick }: { side: 'left' | 'right'; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'absolute top-1/2 z-10 flex size-10 -translate-y-1/2 items-center justify-center rounded-full bg-black/30 text-white/80 backdrop-blur transition-colors hover:bg-black/50 hover:text-white',
        side === 'left' ? 'left-3' : 'right-3',
      )}
    >
      {side === 'left' ? <ChevronLeft size={20} /> : <ChevronRight size={20} />}
    </button>
  )
}
