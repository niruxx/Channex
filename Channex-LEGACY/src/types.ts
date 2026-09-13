export type SiteSchema = 'yotsuba' | 'lynxchan'
export type MediaLayout = 'flat-cdn' | 'board-dirs' | 'file-store'
export type PostEngine = 'lynxchan' | 'external'

export interface ChanSite {
  id: string
  name: string
  schema: SiteSchema
  mediaLayout: MediaLayout
  apiOrigin: string
  siteOrigin: string
  mediaOrigin: string
  accent: string
  isCustom: boolean
  nsfw: boolean
  defaultBoards: Board[]
  postEngine: PostEngine
}

export interface Board {
  code: string
  title: string
  description?: string
  nsfw?: boolean
}

export interface PostFile {
  url: string
  thumbUrl: string
  name: string
  ext: string
  width?: number
  height?: number
  thumbWidth?: number
  thumbHeight?: number
  size?: number
  isVideo: boolean
  spoiler?: boolean
}

export interface Post {
  id: string
  threadId: string
  isOp: boolean
  name: string
  tripcode?: string
  capcode?: string
  subject?: string
  timestamp: number
  commentHtml: string
  files: PostFile[]
  sticky?: boolean
  closed?: boolean
  archived?: boolean
  countryCode?: string
  countryName?: string
  replyCount?: number
  imageCount?: number
  omittedPosts?: number
  omittedImages?: number
}

export interface ThreadData {
  op: Post
  replies: Post[]
}

export interface ReplyDraft {
  comment: string
  name?: string
  options?: string
  subject?: string
  spoiler?: boolean
  captchaAnswer?: string
  file?: File
}

export interface ChanAdapter {
  fetchBoards(site: ChanSite): Promise<Board[]>
  fetchCatalog(site: ChanSite, boardCode: string): Promise<Post[]>
  fetchThread(site: ChanSite, boardCode: string, threadId: string): Promise<ThreadData>
  threadWebUrl(site: ChanSite, boardCode: string, threadId: string): string
}

export interface Bookmark {
  siteId: string
  boardCode: string
  threadId: string
  subject?: string
  excerpt?: string
  thumbUrl?: string
  addedAt: number
  lastSeenReplyCount?: number
}

export type DownloadStatus = 'pending' | 'downloading' | 'done' | 'error' | 'skipped'

export interface DownloadItem {
  id: string
  url: string
  fileName: string
  status: DownloadStatus
  error?: string
  bytesTotal?: number
}

export interface DownloadJob {
  id: string
  label: string
  createdAt: number
  destDir: string
  items: DownloadItem[]
}
