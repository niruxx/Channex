import type { Post } from '../../types'

export type CatalogSort = 'bump' | 'replies' | 'images' | 'oldest'

export const CATALOG_SORTS: { value: CatalogSort; label: string }[] = [
  { value: 'bump', label: 'Latest' },
  { value: 'replies', label: 'Most replies' },
  { value: 'images', label: 'Most images' },
  { value: 'oldest', label: 'Oldest' },
]

const COMPARATORS: Record<CatalogSort, (a: Post, b: Post) => number> = {
  bump: (a, b) => b.timestamp - a.timestamp,
  replies: (a, b) => (b.replyCount ?? 0) - (a.replyCount ?? 0),
  images: (a, b) => (b.imageCount ?? 0) - (a.imageCount ?? 0),
  oldest: (a, b) => a.timestamp - b.timestamp,
}

// Stickies always float to the top regardless of sort — that's what they're for.
export function sortThreads(threads: Post[], sort: CatalogSort): Post[] {
  return [...threads].sort((a, b) => {
    if (!!a.sticky !== !!b.sticky) return a.sticky ? -1 : 1
    return COMPARATORS[sort](a, b)
  })
}

export function filterThreads(threads: Post[], query: string, excerptOf: (p: Post) => string): Post[] {
  const q = query.trim().toLowerCase()
  if (!q) return threads
  return threads.filter((t) => {
    const subject = t.subject?.toLowerCase() ?? ''
    if (subject.includes(q)) return true
    if (t.threadId.includes(q)) return true
    return excerptOf(t).toLowerCase().includes(q)
  })
}
