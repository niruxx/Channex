import { create } from 'zustand'
import type { DownloadItem, DownloadJob } from '../types'
import { downloadToFile, ensureDir, uniqueDestPath } from '../lib/download'
import { uid } from '../lib/format'

const CONCURRENCY = 4

interface DownloadsState {
  jobs: DownloadJob[]
  startJob: (label: string, destDir: string, files: Array<{ url: string; fileName: string }>) => Promise<string>
  clearJob: (jobId: string) => void
  clearFinished: () => void
}

function updateItem(jobs: DownloadJob[], jobId: string, itemId: string, patch: Partial<DownloadItem>): DownloadJob[] {
  return jobs.map((j) =>
    j.id !== jobId ? j : { ...j, items: j.items.map((it) => (it.id === itemId ? { ...it, ...patch } : it)) },
  )
}

export const useDownloadsStore = create<DownloadsState>((set) => ({
  jobs: [],

  startJob: async (label, destDir, files) => {
    const jobId = uid()
    const items: DownloadItem[] = files.map((f) => ({ id: uid(), url: f.url, fileName: f.fileName, status: 'pending' }))
    const job: DownloadJob = { id: jobId, label, createdAt: Date.now(), destDir, items }
    set((s) => ({ jobs: [job, ...s.jobs] }))

    try {
      await ensureDir(destDir)
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      set((s) => ({
        jobs: s.jobs.map((j) =>
          j.id !== jobId ? j : { ...j, items: j.items.map((it) => ({ ...it, status: 'error', error: message })) },
        ),
      }))
      return jobId
    }

    let cursor = 0
    async function worker() {
      while (cursor < items.length) {
        const item = items[cursor++]
        set((s) => ({ jobs: updateItem(s.jobs, jobId, item.id, { status: 'downloading' }) }))
        try {
          const dest = await uniqueDestPath(destDir, item.fileName)
          await downloadToFile(item.url, dest)
          set((s) => ({ jobs: updateItem(s.jobs, jobId, item.id, { status: 'done' }) }))
        } catch (e) {
          set((s) => ({
            jobs: updateItem(s.jobs, jobId, item.id, { status: 'error', error: e instanceof Error ? e.message : String(e) }),
          }))
        }
      }
    }
    await Promise.all(Array.from({ length: Math.min(CONCURRENCY, items.length) }, worker))
    return jobId
  },

  clearJob: (jobId) => set((s) => ({ jobs: s.jobs.filter((j) => j.id !== jobId) })),
  clearFinished: () =>
    set((s) => ({
      jobs: s.jobs.filter((j) => j.items.some((it) => it.status === 'pending' || it.status === 'downloading')),
    })),
}))
