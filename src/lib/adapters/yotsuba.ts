import type { Board, ChanAdapter, ChanSite, Post, ThreadData } from '../../types'
import { getJson } from '../http'
import { decodeEntities, sanitizeCommentHtml } from '../sanitize'

const VIDEO_EXTS = new Set(['.webm', '.mp4', '.mov'])

interface RawPost {
  no: number
  resto: number
  sticky?: number
  closed?: number
  archived?: number
  time: number
  name?: string
  trip?: string
  capcode?: string
  country?: string
  country_name?: string
  sub?: string
  com?: string
  tim?: number | string
  filename?: string
  ext?: string
  fsize?: number
  w?: number
  h?: number
  tn_w?: number
  tn_h?: number
  replies?: number
  images?: number
  omitted_posts?: number
  omitted_images?: number
  spoiler?: number
  filedeleted?: number
}

function mediaUrls(site: ChanSite, board: string, tim: number | string, ext: string) {
  const t = String(tim)
  switch (site.mediaLayout) {
    case 'flat-cdn':
      return {
        url: `${site.mediaOrigin}/${board}/${t}${ext}`,
        thumbUrl: `${site.mediaOrigin}/${board}/${t}s.jpg`,
      }
    case 'file-store':
      return {
        url: `${site.mediaOrigin}/file_store/${t}${ext}`,
        thumbUrl: `${site.mediaOrigin}/file_store/thumb/${t}.jpg`,
      }
    case 'board-dirs':
    default:
      return {
        url: `${site.mediaOrigin}/${board}/src/${t}${ext}`,
        thumbUrl: `${site.mediaOrigin}/${board}/thumb/${t}.jpg`,
      }
  }
}

function normalizePost(raw: RawPost, site: ChanSite, board: string): Post {
  const isOp = raw.resto === 0
  const files = []
  if (raw.tim && raw.ext && !raw.filedeleted) {
    const { url, thumbUrl } = mediaUrls(site, board, raw.tim, raw.ext)
    files.push({
      url,
      thumbUrl,
      name: `${raw.filename ?? raw.tim}${raw.ext}`,
      ext: raw.ext,
      width: raw.w,
      height: raw.h,
      thumbWidth: raw.tn_w,
      thumbHeight: raw.tn_h,
      size: raw.fsize,
      isVideo: VIDEO_EXTS.has(raw.ext.toLowerCase()),
      spoiler: !!raw.spoiler,
    })
  }
  return {
    id: String(raw.no),
    threadId: String(isOp ? raw.no : raw.resto),
    isOp,
    name: decodeEntities(raw.name || 'Anonymous'),
    tripcode: raw.trip ? decodeEntities(raw.trip) : undefined,
    capcode: raw.capcode,
    subject: raw.sub ? decodeEntities(raw.sub) : undefined,
    timestamp: raw.time * 1000,
    commentHtml: sanitizeCommentHtml(raw.com || ''),
    files,
    sticky: !!raw.sticky,
    closed: !!raw.closed,
    archived: !!raw.archived,
    countryCode: raw.country,
    countryName: raw.country_name,
    replyCount: raw.replies,
    imageCount: raw.images,
    omittedPosts: raw.omitted_posts,
    omittedImages: raw.omitted_images,
  }
}

async function fetchBoards(site: ChanSite): Promise<Board[]> {
  try {
    const data = await getJson<unknown>(`${site.apiOrigin}/boards.json`)
    if (Array.isArray(data)) {
      return data.map((b: any) => ({
        code: b.uri ?? b.board,
        title: b.title ?? b.boardName ?? b.uri,
        description: b.subtitle ?? b.meta_description,
      }))
    }
    if (data && typeof data === 'object' && Array.isArray((data as any).boards)) {
      return (data as any).boards.map((b: any) => ({
        code: b.board,
        title: b.title,
        description: b.meta_description,
      }))
    }
    return site.defaultBoards
  } catch {
    return site.defaultBoards
  }
}

async function fetchCatalog(site: ChanSite, boardCode: string): Promise<Post[]> {
  const pages = await getJson<Array<{ page: number; threads: RawPost[] }>>(
    `${site.apiOrigin}/${boardCode}/catalog.json`,
  )
  const threads: Post[] = []
  for (const page of pages) {
    for (const t of page.threads) {
      threads.push(normalizePost(t, site, boardCode))
    }
  }
  return threads
}

async function fetchThread(site: ChanSite, boardCode: string, threadId: string): Promise<ThreadData> {
  const data = await getJson<{ posts: RawPost[] }>(
    `${site.apiOrigin}/${boardCode}/thread/${threadId}.json`,
  )
  const [opRaw, ...replyRaw] = data.posts
  return {
    op: normalizePost(opRaw, site, boardCode),
    replies: replyRaw.map((r) => normalizePost(r, site, boardCode)),
  }
}

function threadWebUrl(site: ChanSite, boardCode: string, threadId: string): string {
  if (site.mediaLayout === 'flat-cdn') {
    return `${site.siteOrigin}/${boardCode}/thread/${threadId}`
  }
  return `${site.siteOrigin}/${boardCode}/res/${threadId}.html`
}

export const yotsubaAdapter: ChanAdapter = {
  fetchBoards,
  fetchCatalog,
  fetchThread,
  threadWebUrl,
}
