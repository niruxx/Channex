import { motion } from 'motion/react'
import { CheckCircle2, Download, FolderOpen, Loader2, Trash2, XCircle } from 'lucide-react'
import { useDownloadsStore } from '../../store/useDownloadsStore'
import { revealInFolder } from '../../lib/download'
import { EmptyState } from '../common/EmptyState'
import { cn } from '../../lib/cn'

export function DownloadsPanel() {
  const jobs = useDownloadsStore((s) => s.jobs)
  const clearJob = useDownloadsStore((s) => s.clearJob)
  const clearFinished = useDownloadsStore((s) => s.clearFinished)

  if (jobs.length === 0) {
    return <EmptyState icon={Download} title="No downloads yet" description="Files you save from threads or the media viewer will show up here with live progress." />
  }

  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <div className="mx-auto max-w-2xl">
        <div className="mb-6 flex items-center gap-3">
          <div>
            <h1 className="text-xl font-semibold text-ink">Downloads</h1>
            <p className="text-sm text-ink-faint">{jobs.length} job{jobs.length === 1 ? '' : 's'}</p>
          </div>
          <button type="button" onClick={clearFinished} className="btn-ghost ml-auto px-2.5 py-1.5 text-xs">
            Clear finished
          </button>
        </div>
        <div className="flex flex-col gap-3">
          {jobs.map((job, i) => {
            const done = job.items.filter((it) => it.status === 'done').length
            const errored = job.items.filter((it) => it.status === 'error').length
            const inProgress = job.items.some((it) => it.status === 'pending' || it.status === 'downloading')
            return (
              <motion.div
                key={job.id}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.25, delay: Math.min(i * 0.02, 0.2) }}
                className="card p-4"
              >
                <div className="flex items-center gap-2.5">
                  {inProgress ? (
                    <Loader2 size={15} className="animate-spin text-accent" />
                  ) : errored > 0 ? (
                    <XCircle size={15} className="text-red-400" />
                  ) : (
                    <CheckCircle2 size={15} className="text-emerald-400" />
                  )}
                  <span className="truncate text-sm font-medium text-ink">{job.label}</span>
                  <span className="ml-auto shrink-0 text-xs text-ink-faint">
                    {done}/{job.items.length}
                    {errored > 0 && ` · ${errored} failed`}
                  </span>
                  <button type="button" onClick={() => revealInFolder(job.destDir)} title="Open folder" className="btn-icon size-7">
                    <FolderOpen size={13} />
                  </button>
                  <button type="button" onClick={() => clearJob(job.id)} title="Remove" className="btn-icon size-7">
                    <Trash2 size={13} />
                  </button>
                </div>
                <div className="mt-3 h-1.5 overflow-hidden rounded-full bg-surface-3">
                  <motion.div
                    className={cn('h-full rounded-full', errored > 0 && !inProgress ? 'bg-red-400' : 'bg-accent')}
                    initial={false}
                    animate={{ width: `${(done / Math.max(job.items.length, 1)) * 100}%` }}
                    transition={{ duration: 0.3 }}
                  />
                </div>
              </motion.div>
            )
          })}
        </div>
      </div>
    </div>
  )
}
