import type { Board, ChanAdapter, ChanSite, Post, PostFile, ThreadData } from '../../types'
import { getJson } from '../http'
import { sanitizeCommentHtml } from '../sanitize'

interface RawFile {
  originalName?: string
  path: string
  thumb?: string
  mime?: string
  size?: number
  width?: number
  height?: number
}

interface RawCatalogThread {
  threadId: number
  subject?: string
  message?: string
  markdown?: string
  postCount?: number
  fileCount?: number
  thumb?: string
  mime?: string
  lastBump?: string
  locked?: boolean
  pinned?: boolean
  cyclic?: boolean
  autoSage?: boolean
}

interface RawPost {
  postId: number
  message?: string
  markdown?: string
  name?: string
  signedRole?: string
  creation?: string
  files?: RawFile[]
}

interface RawThread extends RawPost {
  threadId: number
  subject?: string
  locked?: boolean
  pinned?: boolean
  cyclic?: boolean
  archived?: boolean
  posts?: RawPost[]
}

function resolveUrl(site: ChanSite, path?: string): string {
  if (!path) return ''
  if (/^https?:\/\//i.test(path)) return path
  return `${site.mediaOrigin}${path.startsWith('/') ? '' : '/'}${path}`
}

function extFromPath(path?: string): string {
  if (!path) return ''
  const m = /\.[a-zA-Z0-9]+$/.exec(path)
  return m ? m[0] : ''
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

function commentHtmlOf(raw: { markdown?: string; message?: string }): string {
  if (raw.markdown) return sanitizeCommentHtml(raw.markdown)
  if (raw.message) return sanitizeCommentHtml(`<p>${escapeHtml(raw.message)}</p>`)
  return ''
}

function fileOf(site: ChanSite, f: RawFile): PostFile {
  const url = resolveUrl(site, f.path)
  const thumbUrl = resolveUrl(site, f.thumb) || url
  return {
    url,
    thumbUrl,
    name: f.originalName || f.path.split('/').pop() || 'file',
    ext: extFromPath(f.originalName || f.path),
    width: f.width,
    height: f.height,
    size: f.size,
    isVideo: (f.mime || '').startsWith('video/'),
  }
}

function normalizeCatalogThread(raw: RawCatalogThread, site: ChanSite): Post {
  const thumbUrl = resolveUrl(site, raw.thumb)
  const files: PostFile[] = raw.thumb
    ? [
        {
          url: thumbUrl,
          thumbUrl,
          name: 'thumbnail',
          ext: extFromPath(raw.thumb),
          isVideo: (raw.mime || '').startsWith('video/'),
        },
      ]
    : []
  return {
    id: String(raw.threadId),
    threadId: String(raw.threadId),
    isOp: true,
    name: 'Anonymous',
    subject: raw.subject,
    timestamp: raw.lastBump ? Date.parse(raw.lastBump) : Date.now(),
    commentHtml: commentHtmlOf(raw),
    files,
    sticky: !!raw.pinned,
    closed: !!raw.locked,
    replyCount: Math.max(0, (raw.postCount ?? 1) - 1),
    imageCount: raw.fileCount,
  }
}

function normalizePost(raw: RawPost, site: ChanSite, threadId: string, isOp: boolean, threadMeta?: RawThread): Post {
  return {
    id: String(raw.postId ?? threadId),
    threadId,
    isOp,
    name: raw.name || 'Anonymous',
    capcode: raw.signedRole,
    subject: isOp ? threadMeta?.subject : undefined,
    timestamp: raw.creation ? Date.parse(raw.creation) : Date.now(),
    commentHtml: commentHtmlOf(raw),
    files: (raw.files ?? []).map((f) => fileOf(site, f)),
    sticky: isOp ? !!threadMeta?.pinned : undefined,
    closed: isOp ? !!threadMeta?.locked : undefined,
    archived: isOp ? !!threadMeta?.archived : undefined,
  }
}

async function fetchBoards(site: ChanSite): Promise<Board[]> {
  try {
    const data = await getJson<unknown>(`${site.siteOrigin}/boards.json`)
    if (Array.isArray(data)) {
      return data.map((b: any) => ({
        code: b.uri ?? b.boardUri,
        title: b.title ?? b.boardName,
        description: b.subtitle,
      }))
    }
  } catch {
    // fall through
  }
  try {
    const idx = await getJson<{ topBoards?: Array<{ boardUri: string; boardName: string }> }>(
      `${site.siteOrigin}/index.json`,
    )
    if (idx.topBoards?.length) {
      return idx.topBoards.map((b) => ({ code: b.boardUri, title: b.boardName }))
    }
  } catch {
    // fall through
  }
  return site.defaultBoards
}

async function fetchCatalog(site: ChanSite, boardCode: string): Promise<Post[]> {
  const data = await getJson<RawCatalogThread[]>(`${site.apiOrigin}/${boardCode}/catalog.json`)
  return data.map((t) => normalizeCatalogThread(t, site))
}

async function fetchThread(site: ChanSite, boardCode: string, threadId: string): Promise<ThreadData> {
  const data = await getJson<RawThread>(`${site.apiOrigin}/${boardCode}/res/${threadId}.json`)
  const op = normalizePost(
    { postId: data.postId ?? data.threadId, message: data.message, markdown: data.markdown, name: data.name, signedRole: data.signedRole, creation: data.creation, files: data.files },
    site,
    threadId,
    true,
    data,
  )
  const replies = (data.posts ?? []).map((p) => normalizePost(p, site, threadId, false))
  return { op, replies }
}

function threadWebUrl(site: ChanSite, boardCode: string, threadId: string): string {
  return `${site.siteOrigin}/${boardCode}/res/${threadId}.html`
}

export const lynxchanAdapter: ChanAdapter = {
  fetchBoards,
  fetchCatalog,
  fetchThread,
  threadWebUrl,
}
